use crate::app_data::AppData;
use crate::common::json_error::{database_error, JsonError, ToJsonError};
use crate::models::project::Project;
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
pub(in crate::api::v1) async fn delete_project_handler(
    path: web::Path<i32>, data: Data<AppData>,
) -> Result<HttpResponse, JsonError> {
    let project_id = path.into_inner();

    let mut state = match Project::where_col(|p| p.project_id.equal(project_id))
        .run(&data.db)
        .await
    {
        Ok(rows) => {
            if let Some(s) = rows.into_iter().next() {
                s
            } else {
                return Err("Project not found".to_json_error(StatusCode::NOT_FOUND));
            }
        }
        Err(e) => {
            error!("unable to delete project from database: {}", e);
            return Err(database_error());
        }
    };

    if let Err(e) = state.delete(&data.db).await {
        error!("unable to delete project from database: {}", e);
        return Err(database_error());
    }

    Ok(HttpResponse::Ok().finish())
}
