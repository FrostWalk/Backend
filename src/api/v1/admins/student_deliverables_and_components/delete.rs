use crate::app_data::AppData;
use crate::common::json_error::{error_with_log_id, JsonError, ToJsonError};
use crate::database::repositories::student_deliverables_components_repository;
use actix_web::http::StatusCode;
use actix_web::web::Data;
use actix_web::web::Path;
use actix_web::HttpResponse;

#[utoipa::path(
    delete,
    path = "/v1/admins/student-deliverables-components/{id}",
    responses(
        (status = 200, description = "Student deliverable component relationship deleted successfully"),
        (status = 401, description = "Authentication required", body = JsonError),
        (status = 404, description = "Relationship not found", body = JsonError),
        (status = 500, description = "Internal server error occurred", body = JsonError)
    ),
    security(("AdminAuth" = [])),
    tag = "Student deliverables-components management",
)]
/// Removes a component from a student deliverable.
///
/// This endpoint allows authenticated admins to remove the relationship between a component and a student deliverable by ID.
#[actix_web_grants::protect(any("ROLE_ADMIN_ROOT", "ROLE_ADMIN_PROFESSOR"))]
pub(super) async fn delete_student_deliverable_component_handler(
    path: Path<i32>, data: Data<AppData>,
) -> Result<HttpResponse, JsonError> {
    let id = path.into_inner();

    // Check if the relationship exists
    let relationship_exists = student_deliverables_components_repository::get_by_id(&data.db, id)
        .await
        .map_err(|e| {
            error_with_log_id(
                format!(
                    "unable to load student deliverable component relationship: {}",
                    e
                ),
                "Failed to delete relationship",
                StatusCode::INTERNAL_SERVER_ERROR,
                log::Level::Error,
            )
        })?
        .is_some();

    if !relationship_exists {
        return Err("Relationship not found".to_json_error(StatusCode::NOT_FOUND));
    }

    // Delete the relationship using repository function
    student_deliverables_components_repository::delete_by_id(&data.db, id)
        .await
        .map_err(|e| {
            error_with_log_id(
                format!(
                    "unable to delete student deliverable component relationship: {}",
                    e
                ),
                "Failed to delete relationship",
                StatusCode::INTERNAL_SERVER_ERROR,
                log::Level::Error,
            )
        })?;

    Ok(HttpResponse::Ok().finish())
}
