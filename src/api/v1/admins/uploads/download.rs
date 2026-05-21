use crate::app_data::AppData;
use crate::common::json_error::{error_with_log_id, JsonError, ToJsonError};
use crate::database::repositories::{
    projects_repository, student_deliverable_selections_repository, student_uploads_repository,
};
use crate::jwt::get_user::LoggedUser;
use actix_web::http::header::{ContentDisposition, DispositionParam, DispositionType};
use actix_web::http::StatusCode;
use actix_web::web::{Data, Path};
use actix_web::{HttpMessage, HttpRequest, HttpResponse};

#[utoipa::path(
    get,
    path = "/v1/admins/projects/{project_id}/students/{student_id}/upload",
    params(
        ("project_id" = i32, Path, description = "Project id"),
        ("student_id" = i32, Path, description = "Student id")
    ),
    responses(
        (status = 200, description = "ZIP file downloaded", content_type = "application/zip"),
        (status = 401, description = "Authentication required", body = JsonError),
        (status = 404, description = "Project or upload not found", body = JsonError),
        (status = 500, description = "Internal server error", body = JsonError),
    ),
    security(("AdminAuth" = [])),
    tag = "Student Uploads",
)]
#[actix_web_grants::protect(any("ROLE_ADMIN_ROOT", "ROLE_ADMIN_PROFESSOR"))]
pub(in crate::api::v1) async fn download_student_upload_handler(
    req: HttpRequest, path: Path<(i32, i32)>, data: Data<AppData>,
) -> Result<HttpResponse, JsonError> {
    let _admin = req.extensions().get_admin().map_err(|_| {
        error_with_log_id(
            "entered protected upload download route without loaded admin",
            "Authentication error",
            StatusCode::INTERNAL_SERVER_ERROR,
            log::Level::Error,
        )
    })?;

    let (project_id, student_id) = path.into_inner();
    if projects_repository::get_by_id(&data.db, project_id)
        .await
        .map_err(|e| {
            error_with_log_id(
                format!("failed loading project {}: {}", project_id, e),
                "Database error",
                StatusCode::INTERNAL_SERVER_ERROR,
                log::Level::Error,
            )
        })?
        .is_none()
    {
        return Err("Project not found".to_json_error(StatusCode::NOT_FOUND));
    }

    let selection = student_deliverable_selections_repository::get_by_student_and_project(
        &data.db, student_id, project_id,
    )
    .await
    .map_err(|e| {
        error_with_log_id(
            format!(
                "failed loading selection for student {} and project {}: {}",
                student_id, project_id, e
            ),
            "Database error",
            StatusCode::INTERNAL_SERVER_ERROR,
            log::Level::Error,
        )
    })?
    .ok_or_else(|| {
        "Upload not found for student in project".to_json_error(StatusCode::NOT_FOUND)
    })?;

    let upload = student_uploads_repository::get_by_selection_id(
        &data.db,
        selection.as_ref().student_deliverable_selection_id,
    )
    .await
    .map_err(|e| {
        error_with_log_id(
            format!(
                "failed loading upload for selection {}: {}",
                selection.as_ref().student_deliverable_selection_id,
                e
            ),
            "Database error",
            StatusCode::INTERNAL_SERVER_ERROR,
            log::Level::Error,
        )
    })?
    .ok_or_else(|| {
        "Upload not found for student in project".to_json_error(StatusCode::NOT_FOUND)
    })?;

    let upload_path = upload.as_ref().path.clone();
    let bytes = tokio::fs::read(&upload_path).await.map_err(|e| {
        error_with_log_id(
            format!("failed reading upload file {}: {}", upload_path, e),
            "Stored upload file not available",
            StatusCode::INTERNAL_SERVER_ERROR,
            log::Level::Error,
        )
    })?;

    let filename = format!("{}_{}.zip", project_id, student_id);
    Ok(HttpResponse::Ok()
        .content_type("application/zip")
        .insert_header(ContentDisposition {
            disposition: DispositionType::Attachment,
            parameters: vec![DispositionParam::Filename(filename)],
        })
        .body(bytes))
}
