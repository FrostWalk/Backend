use crate::app_data::AppData;
use crate::common::json_error::{error_with_log_id, JsonError, ToJsonError};
use crate::models::group_deliverable_component::GroupDeliverableComponent;
use actix_web::http::StatusCode;
use actix_web::web::Data;
use actix_web::web::Path;
use actix_web::HttpResponse;

#[utoipa::path(
    delete,
    path = "/v1/admins/group-deliverable-components/{id}",
    responses(
        (status = 200, description = "Group component deleted successfully"),
        (status = 401, description = "Authentication required", body = JsonError),
        (status = 404, description = "Group component not found", body = JsonError),
        (status = 500, description = "Internal server error occurred", body = JsonError)
    ),
    security(("AdminAuth" = [])),
    tag = "Group deliverable components management",
)]
/// Deletes a group component.
///
/// This endpoint allows authenticated admins to remove a group component by ID.
pub(super) async fn delete_group_component_handler(
    path: Path<i32>, data: Data<AppData>,
) -> Result<HttpResponse, JsonError> {
    let id = path.into_inner();

    // Find the existing component by ID
    let mut rows =
        GroupDeliverableComponent::where_col(|gc| gc.group_deliverable_component_id.equal(id))
            .run(&data.db)
            .await
            .map_err(|e| {
                error_with_log_id(
                    format!("unable to load group component: {}", e),
                    "Failed to delete component",
                    StatusCode::INTERNAL_SERVER_ERROR,
                    log::Level::Error,
                )
            })?;

    let mut component_state = match rows.pop() {
        Some(s) => s,
        None => return Err("Group component not found".to_json_error(StatusCode::NOT_FOUND)),
    };

    component_state.delete(&data.db).await.map_err(|e| {
        error_with_log_id(
            format!("unable to delete group component: {}", e),
            "Failed to delete component",
            StatusCode::INTERNAL_SERVER_ERROR,
            log::Level::Error,
        )
    })?;

    Ok(HttpResponse::Ok().finish())
}
