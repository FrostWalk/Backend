use crate::app_data::AppData;
use crate::common::json_error::{database_error, JsonError, ToJsonError};
use crate::models::project::Project;
use actix_web::http::StatusCode;
use actix_web::web::Data;
use actix_web::{web, HttpResponse};
use log::error;
use serde::Serialize;
use utoipa::ToSchema;

#[derive(Debug, Serialize, ToSchema)]
pub(crate) struct GetAllProjectsResponse {
    projects: Vec<Project>,
}
#[utoipa::path(
    get,
    path = "/v1/admins/projects",
    responses(
        (status = 200, description = "Found projects", body = GetAllProjectsResponse),
        (status = 500, description = "Internal server error occurred", body = JsonError)
    ),
    security(("AdminAuth" = [])),
    tag = "Projects management",
)]
/// Get all projects details
pub(in crate::api::v1) async fn get_all_projects_handler(
    data: Data<AppData>,
) -> Result<HttpResponse, JsonError> {
    let states = Project::all().run(&data.db).await.map_err(|e| {
        log::error!("unable to retrieve projects from database: {}", e);
        database_error()
    })?;

    let projects: Vec<Project> = states
        .into_iter()
        .map(welds::state::DbState::into_inner)
        .collect();

    Ok(HttpResponse::Ok().json(GetAllProjectsResponse { projects }))
}

#[utoipa::path(
    get,
    path = "/v1/admins/projects/{id}",
    responses(
        (status = 200, description = "Found project", body = Project),
        (status = 404, description = "project not found", body = JsonError),
        (status = 500, description = "Internal server error", body = JsonError)
    ),
    security(("AdminAuth" = [])),
    tag = "Projects management",
)]
/// Get project details by id
pub(in crate::api::v1) async fn get_one_project_handler(
    path: web::Path<i32>, data: Data<AppData>,
) -> Result<HttpResponse, JsonError> {
    let id = path.into_inner();

    let mut rows = Project::where_col(|p| p.project_id.equal(id))
        .run(&data.db)
        .await
        .map_err(|e| {
            error!("db error: {e}");
            database_error()
        })?;

    let proj = match rows.pop() {
        Some(state) => welds::state::DbState::into_inner(state),
        None => return Err("project not found".to_json_error(StatusCode::NOT_FOUND)),
    };

    Ok(HttpResponse::Ok().json(proj))
}
