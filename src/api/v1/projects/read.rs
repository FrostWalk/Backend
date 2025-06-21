use crate::app_data::AppData;
use crate::common::json_error::{JsonError, ToJsonError};
use crate::database::repository_methods_trait::RepositoryMethods;
use actix_web::http::StatusCode;
use actix_web::web::Data;
use actix_web::{web, HttpResponse};
use entity::projects::Model;
use log::error;
use serde::Serialize;
use utoipa::ToSchema;

#[derive(Debug, Serialize, ToSchema)]
pub(crate) struct GetAllProjectsResponse {
    projects: Vec<Model>,
}
#[utoipa::path(
    get,
    path = "/v1/projects",
    responses(
        (status = 200, description = "Found projects", body = GetAllProjectsResponse),
        (status = 500, description = "Internal server error occurred", body = JsonError)
    ),
    security(("AdminAuth" = []), ("UserAuth"=[])),
    tag = "Projects management",
)]
pub(super) async fn get_all_projects_handler(
    data: Data<AppData>,
) -> Result<HttpResponse, JsonError> {
    let projects = match data.repositories.projects.get_all().await {
        Ok(p) => p,
        Err(e) => {
            error!("unable to retrieve projects from database: {}", e);
            return Err("database error".to_json_error(StatusCode::INTERNAL_SERVER_ERROR));
        }
    };

    Ok(HttpResponse::Ok().json(GetAllProjectsResponse { projects }))
}

#[utoipa::path(
    get,
    path = "/v1/projects/{id}",
    responses(
        (status = 200, description = "Found project", body = Model),
        (status = 404, description = "project not found", body = JsonError),
        (status = 500, description = "Internal server error", body = JsonError)
    ),
    security(("AdminAuth" = []), ("UserAuth"=[])),
    tag = "Projects management",
)]
pub(super) async fn get_one_project_handler(
    path: web::Path<i32>, data: Data<AppData>,
) -> Result<HttpResponse, JsonError> {
    let id = path.into_inner();

    match data.repositories.projects.get_from_id(id).await {
        Ok(o) => match o {
            None => Err("project not found".to_json_error(StatusCode::NOT_FOUND)),
            Some(p) => Ok(HttpResponse::Ok().json(p)),
        },
        Err(e) => {
            error!("unable to retrieve project from database: {}", e);
            Err("database error".to_json_error(StatusCode::INTERNAL_SERVER_ERROR))
        }
    }
}
