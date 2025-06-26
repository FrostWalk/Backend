use crate::app_data::AppData;
use crate::common::json_error::{JsonError, ToJsonError};
use crate::database::repository_methods_trait::RepositoryMethods;
use actix_web::http::StatusCode;
use actix_web::web::{Data, Json};
use actix_web::HttpResponse;
use chrono::{Datelike, Local};
use entity::projects;
use log::error;
use sea_orm::{NotSet, Set};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Deserialize, ToSchema)]
pub(crate) struct CreateProjectScheme {
    #[schema(example = "Project Name")]
    pub name: String,
    #[schema(example = 10)]
    pub max_student_uploads: i32,
    #[schema(example = 4)]
    pub max_group_size: i32,
    #[schema(example = true)]
    pub active: bool,
}
#[derive(Debug, Serialize, ToSchema)]
pub(crate) struct CreateProjectResponse {
    project_id: i32,
}
#[utoipa::path(
    post,
    path = "/v1/admins/projects",
    request_body = CreateProjectScheme,
    responses(
        (status = 200, description = "Project created successfully", body = CreateProjectResponse),
        (status = 400, description = "Invalid data in request", body = JsonError),
        (status = 500, description = "Internal server error", body = JsonError)
    ),
    security(("AdminAuth" = [])),
    tag = "Projects management",
)]
/// Create a project
pub(in crate::api::v1) async fn create_project_handler(
    payload: Json<CreateProjectScheme>, data: Data<AppData>,
) -> Result<HttpResponse, JsonError> {
    let scheme = payload.into_inner();

    if scheme.name.is_empty() {
        return Err("Name field is mandatory".to_json_error(StatusCode::BAD_REQUEST));
    } else if scheme.max_student_uploads < 1 {
        return Err(
            "Max student uploads must be greater than 0".to_json_error(StatusCode::BAD_REQUEST)
        );
    } else if scheme.max_group_size < 2 {
        return Err("Max group size must be greater than 1".to_json_error(StatusCode::BAD_REQUEST));
    }

    let project = projects::ActiveModel {
        project_id: NotSet,
        name: Set(scheme.name),
        year: Set(Local::now().year()),
        max_student_uploads: Set(scheme.max_student_uploads),
        max_group_size: Set(scheme.max_group_size),
        active: Set(scheme.active),
    };

    let result = match data.repositories.projects.create(project).await {
        Ok(r) => r,
        Err(e) => {
            error!("unable to create project: {}", e);
            return Err(
                "unable to create project scheme".to_json_error(StatusCode::INTERNAL_SERVER_ERROR)
            );
        }
    };

    Ok(HttpResponse::Ok().json(CreateProjectResponse {
        project_id: result.last_insert_id,
    }))
}
