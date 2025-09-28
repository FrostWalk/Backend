use crate::app_data::AppData;
use crate::common::json_error::{database_error, JsonError, ToJsonError};
use crate::models::project::Project;
use actix_web::http::StatusCode;
use actix_web::web::{Data, Json};
use actix_web::HttpResponse;
use chrono::{Datelike, Local};
use log::error;
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
        (status = 201, description = "Project created successfully", body = CreateProjectResponse),
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

    let mut p = Project::new();
    p.name = scheme.name;
    p.year = Local::now().year();
    p.max_student_uploads = scheme.max_student_uploads;
    p.max_group_size = scheme.max_group_size;
    p.active = scheme.active;

    match p.save(&data.db).await {
        Ok(_) => {}
        Err(e) => {
            error!("unable to insert project {:?} in database: {}", p, e);
            return Err(database_error());
        }
    }

    Ok(HttpResponse::Created().json(CreateProjectResponse {
        project_id: p.project_id,
    }))
}
