use crate::app_data::AppData;
use crate::common::json_error::{database_error, JsonError, ToJsonError};
use crate::database::repository_methods_trait::RepositoryMethods;
use actix_web::http::StatusCode;
use actix_web::web::Data;
use actix_web::{web, HttpResponse};
use log::error;

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
pub(super) async fn delete_project_handler(
    path: web::Path<i32>, data: Data<AppData>,
) -> Result<HttpResponse, JsonError> {
    let project_id = path.into_inner();

    let deleted = match data.repositories.projects.delete_from_id(project_id).await {
        Ok(d) => d.rows_affected,
        Err(e) => {
            error!("unable to delete project from database {}", e);
            return Err(database_error());
        }
    };

    if deleted == 0 {
        return Err("project not found".to_json_error(StatusCode::NOT_FOUND));
    }

    Ok(HttpResponse::Ok().finish())
}
