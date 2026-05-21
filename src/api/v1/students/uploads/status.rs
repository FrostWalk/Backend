use crate::app_data::AppData;
use crate::common::json_error::{error_with_log_id, JsonError, ToJsonError};
use crate::database::repositories::{
    projects_repository, student_deliverable_selections_repository, student_uploads_repository,
};
use crate::jwt::get_user::LoggedUser;
use actix_web::http::StatusCode;
use actix_web::web::{Data, Path};
use actix_web::{HttpMessage, HttpRequest, HttpResponse};
use chrono::{DateTime, Utc};
use serde::Serialize;
use utoipa::ToSchema;
use welds::state::DbState;

#[derive(Debug, Serialize, ToSchema)]
pub(crate) struct StudentUploadStatusResponse {
    pub upload_id: Option<i32>,
    pub upload_count: i32,
    pub uploads_remaining: i32,
    pub timestamp: Option<DateTime<Utc>>,
    pub upload_deadline: Option<DateTime<Utc>>,
}

#[utoipa::path(
    get,
    path = "/v1/students/projects/{project_id}/upload",
    params(
        ("project_id" = i32, Path, description = "Project id")
    ),
    responses(
        (status = 200, description = "Upload status", body = StudentUploadStatusResponse),
        (status = 401, description = "Authentication required", body = JsonError),
        (status = 404, description = "Project or deliverable selection not found", body = JsonError),
        (status = 500, description = "Internal server error", body = JsonError),
    ),
    security(("StudentAuth" = [])),
    tag = "Student Uploads",
)]
#[actix_web_grants::protect("ROLE_STUDENT")]
pub(in crate::api::v1) async fn get_upload_status_handler(
    req: HttpRequest, path: Path<i32>, data: Data<AppData>,
) -> Result<HttpResponse, JsonError> {
    let project_id = path.into_inner();
    let student = req.extensions().get_student().map_err(|_| {
        error_with_log_id(
            "entered protected upload status route without loaded student",
            "Authentication error",
            StatusCode::INTERNAL_SERVER_ERROR,
            log::Level::Error,
        )
    })?;

    let project_state = projects_repository::get_by_id(&data.db, project_id)
        .await
        .map_err(|e| {
            error_with_log_id(
                format!("failed loading project {}: {}", project_id, e),
                "Database error",
                StatusCode::INTERNAL_SERVER_ERROR,
                log::Level::Error,
            )
        })?
        .ok_or_else(|| "Project not found".to_json_error(StatusCode::NOT_FOUND))?;
    let project = DbState::into_inner(project_state);

    let selection_state = student_deliverable_selections_repository::get_by_student_and_project(
        &data.db,
        student.student_id,
        project_id,
    )
    .await
    .map_err(|e| {
        error_with_log_id(
            format!(
                "failed loading deliverable selection for student {} project {}: {}",
                student.student_id, project_id, e
            ),
            "Database error",
            StatusCode::INTERNAL_SERVER_ERROR,
            log::Level::Error,
        )
    })?
    .ok_or_else(|| {
        "Student has no deliverable selection for this project".to_json_error(StatusCode::NOT_FOUND)
    })?;
    let selection = DbState::into_inner(selection_state);

    let upload_state = student_uploads_repository::get_by_selection_id(
        &data.db,
        selection.student_deliverable_selection_id,
    )
    .await
    .map_err(|e| {
        error_with_log_id(
            format!(
                "failed loading upload for selection {}: {}",
                selection.student_deliverable_selection_id, e
            ),
            "Database error",
            StatusCode::INTERNAL_SERVER_ERROR,
            log::Level::Error,
        )
    })?;

    if let Some(upload_state) = upload_state {
        let upload = DbState::into_inner(upload_state);
        Ok(HttpResponse::Ok().json(StudentUploadStatusResponse {
            upload_id: Some(upload.upload_id),
            upload_count: upload.upload_count,
            uploads_remaining: (project.max_student_uploads - upload.upload_count).max(0),
            timestamp: Some(upload.timestamp),
            upload_deadline: project.upload_deadline,
        }))
    } else {
        Ok(HttpResponse::Ok().json(StudentUploadStatusResponse {
            upload_id: None,
            upload_count: 0,
            uploads_remaining: project.max_student_uploads,
            timestamp: None,
            upload_deadline: project.upload_deadline,
        }))
    }
}
