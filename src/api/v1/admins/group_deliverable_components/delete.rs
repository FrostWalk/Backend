use crate::app_data::AppData;
use crate::common::json_error::{error_with_log_id, JsonError, ToJsonError};
use crate::database::repositories::group_deliverable_components_repository;
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
#[actix_web_grants::protect(any("ROLE_ADMIN_ROOT", "ROLE_ADMIN_PROFESSOR"))]
pub(super) async fn delete_group_component_handler(
    path: Path<i32>, data: Data<AppData>,
) -> Result<HttpResponse, JsonError> {
    let id = path.into_inner();

    // Check if the component exists
    let component_exists = group_deliverable_components_repository::get_by_id(&data.db, id)
        .await
        .map_err(|e| {
            error_with_log_id(
                format!("unable to load group component: {}", e),
                "Failed to delete component",
                StatusCode::INTERNAL_SERVER_ERROR,
                log::Level::Error,
            )
        })?
        .is_some();

    if !component_exists {
        return Err("Group component not found".to_json_error(StatusCode::NOT_FOUND));
    }

    // Delete the component using repository function
    group_deliverable_components_repository::delete_by_id(&data.db, id)
        .await
        .map_err(|e| {
            error_with_log_id(
                format!("unable to delete group component: {}", e),
                "Failed to delete component",
                StatusCode::INTERNAL_SERVER_ERROR,
                log::Level::Error,
            )
        })?;

    Ok(HttpResponse::Ok().finish())
}
