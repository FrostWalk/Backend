use crate::app_data::AppData;
use crate::common::json_error::{error_with_log_id_and_payload, JsonError, ToJsonError};
use crate::models::group_deliverable_component::GroupDeliverableComponent;
use actix_web::http::StatusCode;
use actix_web::web::Path;
use actix_web::web::{Data, Json};
use actix_web::HttpResponse;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub(crate) struct UpdateGroupComponentScheme {
    #[schema(example = "Updated Resistor")]
    pub name: String,
    #[schema(example = "true")]
    pub sellable: bool,
}

#[utoipa::path(
    patch,
    path = "/v1/admins/group-deliverable-components/{id}",
    request_body = UpdateGroupComponentScheme,
    responses(
        (status = 200, description = "Component updated successfully"),
        (status = 400, description = "Invalid data in request", body = JsonError),
        (status = 401, description = "Authentication required", body = JsonError),
        (status = 404, description = "Group component not found", body = JsonError),
        (status = 409, description = "Component with this name already exists for the project", body = JsonError),
        (status = 500, description = "Internal server error occurred", body = JsonError)
    ),
    security(("AdminAuth" = [])),
    tag = "Group deliverable components management",
)]
/// Updates a group component.
///
/// This endpoint allows authenticated admins to modify the name of a group component by ID.
pub(super) async fn update_group_component_handler(
    path: Path<i32>, req: Json<UpdateGroupComponentScheme>, data: Data<AppData>,
) -> Result<HttpResponse, JsonError> {
    let id = path.into_inner();

    // Find the existing component by ID
    let mut rows =
        GroupDeliverableComponent::where_col(|gc| gc.group_deliverable_component_id.equal(id))
            .run(&data.db)
            .await
            .map_err(|e| {
                error_with_log_id_and_payload(
                    format!("unable to load group component: {}", e),
                    "Failed to update component",
                    StatusCode::INTERNAL_SERVER_ERROR,
                    log::Level::Error,
                    &req,
                )
            })?;

    let mut component_state = match rows.pop() {
        Some(s) => s,
        None => return Err("Group component not found".to_json_error(StatusCode::NOT_FOUND)),
    };

    // Check if another component with this name already exists for the same project
    let existing =
        GroupDeliverableComponent::where_col(|gc| gc.project_id.equal(component_state.project_id))
            .where_col(|gc| gc.name.equal(&req.name))
            .where_col(|gc| gc.group_deliverable_component_id.not_equal(id))
            .run(&data.db)
            .await
            .map_err(|e| {
                error_with_log_id_and_payload(
                    format!("unable to check existing component: {}", e),
                    "Failed to update component",
                    StatusCode::INTERNAL_SERVER_ERROR,
                    log::Level::Error,
                    &req,
                )
            })?;

    if !existing.is_empty() {
        return Err("Component with this name already exists for the project"
            .to_json_error(StatusCode::CONFLICT));
    }

    // Update the name and sellable
    component_state.name = req.name.clone();
    component_state.sellable = req.sellable;

    component_state.save(&data.db).await.map_err(|e| {
        error_with_log_id_and_payload(
            format!("unable to update group component: {}", e),
            "Failed to update component",
            StatusCode::INTERNAL_SERVER_ERROR,
            log::Level::Error,
            &req,
        )
    })?;

    Ok(HttpResponse::Ok().finish())
}
