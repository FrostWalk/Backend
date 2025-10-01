use crate::app_data::AppData;
use crate::common::json_error::{error_with_log_id_and_payload, JsonError, ToJsonError};
use crate::models::project::Project;
use actix_web::http::StatusCode;
use actix_web::web::{Data, Json};
use actix_web::HttpResponse;
use chrono::{DateTime, Datelike, Local, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub(crate) struct CreateProjectScheme {
    #[schema(example = "Project Name")]
    pub name: String,
    #[schema(example = 10)]
    pub max_student_uploads: i32,
    #[schema(example = 4)]
    pub max_group_size: i32,
    #[schema(example = 15)]
    pub max_groups: i32,
    #[schema(value_type = Option<String>, example = "2025-12-15T23:59:59Z")]
    pub deliverable_selection_deadline: Option<DateTime<Utc>>,
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
    req: Json<CreateProjectScheme>, data: Data<AppData>,
) -> Result<HttpResponse, JsonError> {
    if req.name.is_empty() {
        return Err("Name field is mandatory".to_json_error(StatusCode::BAD_REQUEST));
    } else if req.max_student_uploads < 1 {
        return Err(
            "Max student uploads must be greater than 0".to_json_error(StatusCode::BAD_REQUEST)
        );
    } else if req.max_group_size < 2 {
        return Err("Max group size must be greater than 1".to_json_error(StatusCode::BAD_REQUEST));
    } else if req.max_groups < 1 {
        return Err("Max groups must be greater than 0".to_json_error(StatusCode::BAD_REQUEST));
    }

    let mut p = Project::new();
    p.name = req.name.clone();
    p.year = Local::now().year();
    p.max_student_uploads = req.max_student_uploads;
    p.max_group_size = req.max_group_size;
    p.max_groups = req.max_groups;
    p.deliverable_selection_deadline = req.deliverable_selection_deadline;
    p.active = req.active;

    match p.save(&data.db).await {
        Ok(_) => {}
        Err(e) => {
            return Err(error_with_log_id_and_payload(
                format!("unable to insert project {:?} in database: {}", p, e),
                "Failed to create project",
                StatusCode::INTERNAL_SERVER_ERROR,
                log::Level::Error,
                &req,
            ));
        }
    }

    Ok(HttpResponse::Created().json(CreateProjectResponse {
        project_id: p.project_id,
    }))
}
