use crate::app_data::AppData;
use crate::common::json_error::{error_with_log_id, JsonError, ToJsonError};
use crate::database::repositories::projects_repository;
use actix_web::http::StatusCode;
use actix_web::web::{Data, Path};
use actix_web::HttpResponse;

#[utoipa::path(
    delete,
    path = "/v1/admins/projects/{id}",
    responses(
        (status = 200, description = "Project deleted successfully"),
        (status = 404, description = "Project not found", body = JsonError),
        (status = 500, description = "Internal server error occurred", body = JsonError)
    ),
    security(("AdminAuth" = [])),
    tag = "Projects management",
)]
/// Delete a project by id
pub(in crate::api::v1) async fn delete_project_handler(
    path: Path<i32>, 
    data: Data<AppData>,
) -> Result<HttpResponse, JsonError> {
    let project_id = path.into_inner();

    let deleted = projects_repository::delete_by_id(&data.db, project_id)
        .await
        .map_err(|e| {
            error_with_log_id(
                format!("unable to delete project from database: {}", e),
                "Failed to delete project",
                StatusCode::INTERNAL_SERVER_ERROR,
                log::Level::Error,
            )
        })?;

    if !deleted {
        return Err("Project not found".to_json_error(StatusCode::NOT_FOUND));
    }

    Ok(HttpResponse::Ok().finish())
}
