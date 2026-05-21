use crate::app_data::AppData;
use crate::common::json_error::{error_with_log_id_and_payload, JsonError, ToJsonError};
use crate::database::repositories::{
    fairs_repository, group_component_implementation_details_repository,
    group_deliverable_components_repository, group_deliverable_selections_repository,
    groups_repository, transactions_repository,
};
use crate::jwt::get_user::LoggedUser;
use crate::models::transaction::Transaction;
use actix_web::http::StatusCode;
use actix_web::web::{Data, Json, Path};
use actix_web::HttpMessage;
use actix_web::HttpRequest;
use actix_web::HttpResponse;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub(crate) struct PurchaseRequest {
    #[schema(example = 1)]
    pub buyer_group_id: i32,
    #[schema(example = 2)]
    pub seller_group_deliverable_selection_id: i32,
    #[schema(example = 3)]
    pub group_deliverable_component_id: i32,
}

#[derive(Debug, Serialize, ToSchema)]
pub(crate) struct PurchaseResponse {
    pub transaction_id: i32,
}

#[utoipa::path(
    post,
    path = "/v1/students/fairs/{fair_id}/transactions",
    params(("fair_id" = i32, Path, description = "Fair ID")),
    request_body = PurchaseRequest,
    responses(
        (status = 201, description = "Purchase recorded", body = PurchaseResponse),
        (status = 400, description = "Validation error", body = JsonError),
        (status = 403, description = "Not a group leader or fair not active", body = JsonError),
        (status = 404, description = "Fair, selection, or component not found", body = JsonError),
        (status = 409, description = "Already purchased this component from this group", body = JsonError),
        (status = 500, description = "Internal server error", body = JsonError),
    ),
    security(("StudentAuth" = [])),
    tag = "Fair transactions",
)]
#[actix_web_grants::protect("ROLE_STUDENT")]
pub(in crate::api::v1) async fn purchase_handler(
    req: HttpRequest, path: Path<i32>, body: Json<PurchaseRequest>, data: Data<AppData>,
) -> Result<HttpResponse, JsonError> {
    let fair_id = path.into_inner();
    let student = req.extensions().get_student().map_err(|e| {
        error_with_log_id_and_payload(
            format!("Failed to extract student: {}", e),
            "Authentication error",
            StatusCode::UNAUTHORIZED,
            log::Level::Warn,
            &body,
        )
    })?;

    let fair_state = fairs_repository::get_by_id(&data.db, fair_id)
        .await
        .map_err(|e| {
            error_with_log_id_and_payload(
                format!("DB error fetching fair {}: {}", fair_id, e),
                "Failed to fetch fair",
                StatusCode::INTERNAL_SERVER_ERROR,
                log::Level::Error,
                &body,
            )
        })?
        .ok_or_else(|| "Fair not found".to_json_error(StatusCode::NOT_FOUND))?;

    if !fairs_repository::is_active(&fair_state) {
        return Err("The fair is not currently active".to_json_error(StatusCode::FORBIDDEN));
    }

    let is_leader =
        groups_repository::is_group_leader(&data.db, student.student_id, body.buyer_group_id)
            .await
            .map_err(|e| {
                error_with_log_id_and_payload(
                    format!("DB error checking group leader: {}", e),
                    "Failed to verify group leader status",
                    StatusCode::INTERNAL_SERVER_ERROR,
                    log::Level::Error,
                    &body,
                )
            })?;

    if !is_leader {
        return Err(
            "You must be a group leader to make purchases".to_json_error(StatusCode::FORBIDDEN)
        );
    }

    let buyer_group = groups_repository::get_by_id(&data.db, body.buyer_group_id)
        .await
        .map_err(|e| {
            error_with_log_id_and_payload(
                format!("DB error fetching buyer group: {}", e),
                "Failed to fetch buyer group",
                StatusCode::INTERNAL_SERVER_ERROR,
                log::Level::Error,
                &body,
            )
        })?
        .ok_or_else(|| "Buyer group not found".to_json_error(StatusCode::NOT_FOUND))?;

    if buyer_group.project_id != fair_state.project_id {
        return Err("Buyer group does not belong to the fair's project"
            .to_json_error(StatusCode::BAD_REQUEST));
    }

    let seller_selection =
        group_deliverable_selections_repository::get_by_group_deliverable_selection_id(
            &data.db,
            body.seller_group_deliverable_selection_id,
        )
        .await
        .map_err(|e| {
            error_with_log_id_and_payload(
                format!("DB error fetching seller selection: {}", e),
                "Failed to fetch seller selection",
                StatusCode::INTERNAL_SERVER_ERROR,
                log::Level::Error,
                &body,
            )
        })?
        .ok_or_else(|| "Seller selection not found".to_json_error(StatusCode::NOT_FOUND))?;

    if seller_selection.group_id == body.buyer_group_id {
        return Err("A group cannot purchase from itself".to_json_error(StatusCode::BAD_REQUEST));
    }

    let seller_group = groups_repository::get_by_id(&data.db, seller_selection.group_id)
        .await
        .map_err(|e| {
            error_with_log_id_and_payload(
                format!("DB error fetching seller group: {}", e),
                "Failed to fetch seller group",
                StatusCode::INTERNAL_SERVER_ERROR,
                log::Level::Error,
                &body,
            )
        })?
        .ok_or_else(|| "Seller group not found".to_json_error(StatusCode::NOT_FOUND))?;

    if seller_group.project_id != fair_state.project_id {
        return Err("Seller group does not belong to the fair's project"
            .to_json_error(StatusCode::BAD_REQUEST));
    }

    let component = group_deliverable_components_repository::get_by_id(
        &data.db,
        body.group_deliverable_component_id,
    )
    .await
    .map_err(|e| {
        error_with_log_id_and_payload(
            format!("DB error fetching component: {}", e),
            "Failed to fetch component",
            StatusCode::INTERNAL_SERVER_ERROR,
            log::Level::Error,
            &body,
        )
    })?
    .ok_or_else(|| "Component not found".to_json_error(StatusCode::NOT_FOUND))?;

    if !component.sellable {
        return Err(
            "This component is not marked as sellable".to_json_error(StatusCode::BAD_REQUEST)
        );
    }

    let has_impl = group_component_implementation_details_repository::exists(
        &data.db,
        body.seller_group_deliverable_selection_id,
        body.group_deliverable_component_id,
    )
    .await
    .map_err(|e| {
        error_with_log_id_and_payload(
            format!("DB error checking implementation detail: {}", e),
            "Failed to verify component implementation",
            StatusCode::INTERNAL_SERVER_ERROR,
            log::Level::Error,
            &body,
        )
    })?;

    if !has_impl {
        return Err(
            "The seller group has not published an implementation for this component"
                .to_json_error(StatusCode::BAD_REQUEST),
        );
    }

    let already_purchased = transactions_repository::purchase_exists(
        &data.db,
        body.buyer_group_id,
        body.seller_group_deliverable_selection_id,
        body.group_deliverable_component_id,
    )
    .await
    .map_err(|e| {
        error_with_log_id_and_payload(
            format!("DB error checking duplicate purchase: {}", e),
            "Failed to check purchase",
            StatusCode::INTERNAL_SERVER_ERROR,
            log::Level::Error,
            &body,
        )
    })?;

    if already_purchased {
        return Err(
            "Your group has already purchased this component from this seller"
                .to_json_error(StatusCode::CONFLICT),
        );
    }

    let transaction = Transaction {
        transaction_id: 0,
        buyer_group_id: body.buyer_group_id,
        group_deliverable_selection_id: body.seller_group_deliverable_selection_id,
        group_deliverable_component_id: body.group_deliverable_component_id,
        fair_id,
        timestamp: Utc::now(),
    };

    let created = transactions_repository::create(&data.db, transaction)
        .await
        .map_err(|e| {
            error_with_log_id_and_payload(
                format!("Failed to create transaction: {}", e),
                "Failed to record purchase",
                StatusCode::INTERNAL_SERVER_ERROR,
                log::Level::Error,
                &body,
            )
        })?;

    Ok(HttpResponse::Created().json(PurchaseResponse {
        transaction_id: created.transaction_id,
    }))
}
