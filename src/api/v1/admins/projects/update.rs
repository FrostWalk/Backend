use crate::app_data::AppData;
use crate::common::json_error::{error_with_log_id_and_payload, JsonError, ToJsonError};
use crate::database::repositories::projects_repository;
use actix_web::http::StatusCode;
use actix_web::web::{Data, Json, Path};
use actix_web::HttpResponse;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct UpdateProjectScheme {
    pub name: Option<String>,
    pub max_student_uploads: Option<i32>,
    pub max_group_size: Option<i32>,
    pub active: Option<bool>,
}
#[utoipa::path(
    patch,
    path = "/v1/admins/projects/{id}",
    request_body = UpdateProjectScheme,
    responses(
        (status = 200, description = "Project updated successfully"),
        (status = 400, description = "Invalid data in request", body = JsonError),
        (status = 401, description = "Authentication required", body = JsonError),
        (status = 404, description = "Project not found", body = JsonError),
        (status = 500, description = "Internal server error occurred", body = JsonError)
    ),
    security(("AdminAuth" = [])),
    tag = "Projects management",
)]
/// Update a project details
#[actix_web_grants::protect(any("ROLE_ADMIN_ROOT", "ROLE_ADMIN_PROFESSOR"))]
pub(in crate::api::v1) async fn update_project_handler(
    path: Path<i32>, body: Json<UpdateProjectScheme>, data: Data<AppData>,
) -> Result<HttpResponse, JsonError> {
    let id = path.into_inner();

    // Check if project exists
    let project_exists = projects_repository::get_by_id(&data.db, id)
        .await
        .map_err(|e| {
            error_with_log_id_and_payload(
                format!("unable to load project {}: {}", id, e),
                "Failed to update project",
                StatusCode::INTERNAL_SERVER_ERROR,
                log::Level::Error,
                &body,
            )
        })?
        .is_some();

    if !project_exists {
        return Err("Project not found".to_json_error(StatusCode::NOT_FOUND));
    }

    // Update project using repository function
    projects_repository::update_by_id(
        &data.db,
        id,
        body.name.clone(),
        body.max_student_uploads,
        body.max_group_size,
        body.active,
    )
    .await
    .map_err(|e| {
        error_with_log_id_and_payload(
            format!("unable to update project {}: {}", id, e),
            "Failed to update project",
            StatusCode::INTERNAL_SERVER_ERROR,
            log::Level::Error,
            &body,
        )
    })?;

    Ok(HttpResponse::Ok().finish())
}
