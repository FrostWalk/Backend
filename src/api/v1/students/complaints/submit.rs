use crate::app_data::AppData;
use crate::common::json_error::{error_with_log_id, JsonError, ToJsonError};
use crate::database::repositories::{
    complaints_repository, group_deliverable_selections_repository, groups_repository,
    transactions_repository,
};
use crate::jwt::get_user::LoggedUser;
use crate::models::complaint::Complaint;
use actix_web::http::StatusCode;
use actix_web::web::{Data, Json};
use actix_web::{HttpMessage, HttpRequest, HttpResponse};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use welds::state::DbState;

#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub(crate) struct SubmitComplaintRequest {
    #[schema(example = 1)]
    pub transaction_id: i32,
    #[schema(example = 1)]
    pub from_group_id: i32,
    #[schema(example = "Purchased deliverable missing required documentation.")]
    pub text: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub(crate) struct SubmitComplaintResponse {
    pub complaint_id: i32,
}

#[utoipa::path(
    post,
    path = "/v1/students/complaints",
    request_body = SubmitComplaintRequest,
    responses(
        (status = 201, description = "Complaint submitted", body = SubmitComplaintResponse),
        (status = 400, description = "Validation error", body = JsonError),
        (status = 401, description = "Authentication required", body = JsonError),
        (status = 403, description = "User is not group leader", body = JsonError),
        (status = 404, description = "Transaction or seller group not found", body = JsonError),
        (status = 500, description = "Internal server error", body = JsonError),
    ),
    security(("StudentAuth" = [])),
    tag = "Complaints management",
)]
#[actix_web_grants::protect("ROLE_STUDENT")]
pub(in crate::api::v1) async fn submit_complaint_handler(
    req: HttpRequest, body: Json<SubmitComplaintRequest>, data: Data<AppData>,
) -> Result<HttpResponse, JsonError> {
    let student = req.extensions().get_student().map_err(|_| {
        error_with_log_id(
            "entered a protected route without a student loaded in request",
            "Authentication error",
            StatusCode::INTERNAL_SERVER_ERROR,
            log::Level::Error,
        )
    })?;

    if body.text.trim().is_empty() {
        return Err("Complaint text cannot be empty".to_json_error(StatusCode::BAD_REQUEST));
    }

    let is_leader =
        groups_repository::is_group_leader(&data.db, student.student_id, body.from_group_id)
            .await
            .map_err(|e| {
                error_with_log_id(
                    format!("failed to verify group leadership: {}", e),
                    "Database error",
                    StatusCode::INTERNAL_SERVER_ERROR,
                    log::Level::Error,
                )
            })?;

    if !is_leader {
        return Err("Only group leader can submit complaints".to_json_error(StatusCode::FORBIDDEN));
    }

    let transaction_state = transactions_repository::get_by_id(&data.db, body.transaction_id)
        .await
        .map_err(|e| {
            error_with_log_id(
                format!("failed to fetch transaction {}: {}", body.transaction_id, e),
                "Database error",
                StatusCode::INTERNAL_SERVER_ERROR,
                log::Level::Error,
            )
        })?
        .ok_or_else(|| "Transaction not found".to_json_error(StatusCode::NOT_FOUND))?;

    let transaction = DbState::into_inner(transaction_state);
    if transaction.buyer_group_id != body.from_group_id {
        return Err("Transaction does not belong to provided buyer group"
            .to_json_error(StatusCode::BAD_REQUEST));
    }

    let seller_selection_state =
        group_deliverable_selections_repository::get_by_group_deliverable_selection_id(
            &data.db,
            transaction.group_deliverable_selection_id,
        )
        .await
        .map_err(|e| {
            error_with_log_id(
                format!(
                    "failed to fetch group deliverable selection {}: {}",
                    transaction.group_deliverable_selection_id, e
                ),
                "Database error",
                StatusCode::INTERNAL_SERVER_ERROR,
                log::Level::Error,
            )
        })?
        .ok_or_else(|| "Seller group selection not found".to_json_error(StatusCode::NOT_FOUND))?;

    let seller_selection = DbState::into_inner(seller_selection_state);
    if seller_selection.group_id == body.from_group_id {
        return Err(
            "Cannot submit complaint against same group".to_json_error(StatusCode::BAD_REQUEST)
        );
    }

    let complaint = Complaint {
        complaint_id: 0,
        transaction_id: body.transaction_id,
        from_group_id: body.from_group_id,
        to_group_id: seller_selection.group_id,
        text: body.text.trim().to_string(),
        created_at: Utc::now(),
    };

    let created = complaints_repository::create(&data.db, complaint)
        .await
        .map_err(|e| {
            error_with_log_id(
                format!("failed to create complaint: {}", e),
                "Failed to create complaint",
                StatusCode::INTERNAL_SERVER_ERROR,
                log::Level::Error,
            )
        })?;

    Ok(HttpResponse::Created().json(SubmitComplaintResponse {
        complaint_id: created.complaint_id,
    }))
}
