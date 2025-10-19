use crate::app_data::AppData;
use crate::common::json_error::{error_with_log_id_and_payload, JsonError, ToJsonError};
use crate::models::group_deliverables_component::GroupDeliverablesComponent;
use actix_web::http::StatusCode;
use actix_web::web::Path;
use actix_web::web::{Data, Json};
use actix_web::HttpResponse;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub(crate) struct UpdateGroupDeliverableComponentScheme {
    #[schema(example = "10")]
    pub quantity: i32,
}

#[utoipa::path(
    patch,
    path = "/v1/admins/group-deliverables-components/{id}",
    request_body = UpdateGroupDeliverableComponentScheme,
    responses(
        (status = 200, description = "Group deliverable component relationship updated successfully"),
        (status = 400, description = "Invalid data in request", body = JsonError),
        (status = 401, description = "Authentication required", body = JsonError),
        (status = 404, description = "Relationship not found", body = JsonError),
        (status = 500, description = "Internal server error occurred", body = JsonError)
    ),
    security(("AdminAuth" = [])),
    tag = "Group deliverables-components management",
)]
/// Updates the quantity of a component in a group deliverable.
///
/// This endpoint allows authenticated admins to modify the quantity of a component in a group deliverable by ID.
pub(super) async fn update_group_deliverable_component_handler(
    path: Path<i32>, 
    body: Json<UpdateGroupDeliverableComponentScheme>, 
    data: Data<AppData>,
) -> Result<HttpResponse, JsonError> {
    let id = path.into_inner();
    // Find the existing relationship by ID
    let mut rows = GroupDeliverablesComponent::where_col(|gdc| gdc.id.equal(id))
        .run(&data.db)
        .await
        .map_err(|e| {
            error_with_log_id_and_payload(
                format!(
                    "unable to load group deliverable component relationship: {}",
                    e
                ),
                "Failed to update relationship",
                StatusCode::INTERNAL_SERVER_ERROR,
                log::Level::Error,
                &body,
            )
        })?;

    let mut relationship_state = match rows.pop() {
        Some(s) => s,
        None => return Err("Relationship not found".to_json_error(StatusCode::NOT_FOUND)),
    };

    // Update the quantity
    relationship_state.quantity = body.quantity;

    relationship_state.save(&data.db).await.map_err(|e| {
        error_with_log_id_and_payload(
            format!(
                "unable to update group deliverable component relationship: {}",
                e
            ),
            "Failed to update relationship",
            StatusCode::INTERNAL_SERVER_ERROR,
            log::Level::Error,
            &body,
        )
    })?;

    Ok(HttpResponse::Ok().finish())
}
