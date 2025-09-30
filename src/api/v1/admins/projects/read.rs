use crate::app_data::AppData;
use crate::common::json_error::{error_with_log_id, JsonError, ToJsonError};
use crate::database::repositories::projects_repository;
use crate::models::project::Project;
use actix_web::http::StatusCode;
use actix_web::web::Data;
use actix_web::{web, HttpResponse};
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
    let states = projects_repository::get_all(&data.db).await.map_err(|e| {
        error_with_log_id(
            format!("unable to retrieve projects from database: {}", e),
            "Failed to retrieve projects",
            StatusCode::INTERNAL_SERVER_ERROR,
            log::Level::Error,
        )
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

    let state = projects_repository::get_by_id(&data.db, id)
        .await
        .map_err(|e| {
            error_with_log_id(
                format!("database error: {}", e),
                "Failed to retrieve projects",
                StatusCode::INTERNAL_SERVER_ERROR,
                log::Level::Error,
            )
        })?;

    let proj = match state {
        Some(state) => welds::state::DbState::into_inner(state),
        None => return Err("Project not found".to_json_error(StatusCode::NOT_FOUND)),
    };

    Ok(HttpResponse::Ok().json(proj))
}
