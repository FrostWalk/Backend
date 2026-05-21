use crate::app_data::AppData;
use crate::common::json_error::{error_with_log_id, JsonError, ToJsonError};
use crate::database::repositories::{
    projects_repository, student_deliverable_selections_repository, student_uploads_repository,
};
use crate::jwt::get_user::LoggedUser;
use actix_multipart::Multipart;
use actix_web::http::StatusCode;
use actix_web::web::{Data, Path};
use actix_web::{HttpMessage, HttpRequest, HttpResponse};
use chrono::Utc;
use futures_util::StreamExt;
use serde::Serialize;
use utoipa::ToSchema;
use welds::state::DbState;

const ZIP_MAGIC_BYTES: [u8; 4] = [0x50, 0x4b, 0x03, 0x04];

#[derive(Debug, Serialize, ToSchema)]
pub(crate) struct UploadProjectZipResponse {
    pub upload_id: i32,
    pub upload_count: i32,
    pub uploads_remaining: i32,
}

#[utoipa::path(
    post,
    path = "/v1/students/projects/{project_id}/upload",
    params(
        ("project_id" = i32, Path, description = "Project id")
    ),
    request_body(content = String, description = "Multipart form-data with file field named 'file'", content_type = "multipart/form-data"),
    responses(
        (status = 201, description = "ZIP uploaded successfully", body = UploadProjectZipResponse),
        (status = 400, description = "Validation error", body = JsonError),
        (status = 401, description = "Authentication required", body = JsonError),
        (status = 403, description = "Upload deadline reached", body = JsonError),
        (status = 404, description = "Project or deliverable selection not found", body = JsonError),
        (status = 413, description = "File too large", body = JsonError),
        (status = 500, description = "Internal server error", body = JsonError),
    ),
    security(("StudentAuth" = [])),
    tag = "Student Uploads",
)]
#[actix_web_grants::protect("ROLE_STUDENT")]
pub(in crate::api::v1) async fn upload_project_zip_handler(
    req: HttpRequest, path: Path<i32>, mut payload: Multipart, data: Data<AppData>,
) -> Result<HttpResponse, JsonError> {
    let project_id = path.into_inner();
    let student = req.extensions().get_student().map_err(|_| {
        error_with_log_id(
            "entered protected upload route without loaded student",
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

    if let Some(upload_deadline) = project.upload_deadline {
        if Utc::now() > upload_deadline {
            return Err("Upload deadline has passed".to_json_error(StatusCode::FORBIDDEN));
        }
    }

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

    let current_upload = student_uploads_repository::get_by_selection_id(
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
    let current_count = current_upload
        .as_ref()
        .map(|upload| upload.as_ref().upload_count)
        .unwrap_or(0);

    if current_count >= project.max_student_uploads {
        return Err("Upload attempts exhausted".to_json_error(StatusCode::FORBIDDEN));
    }

    let max_size = data.config.max_upload_size_bytes();
    let mut file_bytes: Option<Vec<u8>> = None;
    while let Some(field_result) = payload.next().await {
        let mut field = field_result.map_err(|e| {
            error_with_log_id(
                format!("failed reading multipart field: {}", e),
                "Invalid multipart data",
                StatusCode::BAD_REQUEST,
                log::Level::Warn,
            )
        })?;
        if field.name() != Some("file") {
            continue;
        }

        let mut bytes = Vec::new();
        let mut current_size: u64 = 0;
        while let Some(chunk_result) = field.next().await {
            let chunk = chunk_result.map_err(|e| {
                error_with_log_id(
                    format!("failed reading multipart chunk: {}", e),
                    "Invalid multipart data",
                    StatusCode::BAD_REQUEST,
                    log::Level::Warn,
                )
            })?;
            current_size += chunk.len() as u64;
            if current_size > max_size {
                return Err("File size exceeds configured maximum"
                    .to_json_error(StatusCode::PAYLOAD_TOO_LARGE));
            }
            bytes.extend_from_slice(&chunk);
        }

        file_bytes = Some(bytes);
        break;
    }

    let file_bytes = file_bytes.ok_or_else(|| {
        "Multipart field 'file' is required".to_json_error(StatusCode::BAD_REQUEST)
    })?;
    if file_bytes.len() < ZIP_MAGIC_BYTES.len() || file_bytes[..4] != ZIP_MAGIC_BYTES {
        return Err(
            "Uploaded file is not a valid ZIP archive".to_json_error(StatusCode::BAD_REQUEST)
        );
    }

    let upload_dir = format!(
        "{}/{}/",
        data.config.uploads_dir().trim_end_matches('/'),
        project_id
    );
    tokio::fs::create_dir_all(&upload_dir).await.map_err(|e| {
        error_with_log_id(
            format!("failed creating upload directory {}: {}", upload_dir, e),
            "Unable to store upload",
            StatusCode::INTERNAL_SERVER_ERROR,
            log::Level::Error,
        )
    })?;

    let file_path = format!("{}{}.zip", upload_dir, student.student_id);
    tokio::fs::write(&file_path, &file_bytes)
        .await
        .map_err(|e| {
            error_with_log_id(
                format!("failed writing upload file {}: {}", file_path, e),
                "Unable to store upload",
                StatusCode::INTERNAL_SERVER_ERROR,
                log::Level::Error,
            )
        })?;

    let saved_state = student_uploads_repository::upsert(
        &data.db,
        selection.student_deliverable_selection_id,
        file_path,
        Utc::now(),
    )
    .await
    .map_err(|e| {
        error_with_log_id(
            format!(
                "failed upserting upload for selection {}: {}",
                selection.student_deliverable_selection_id, e
            ),
            "Failed to store upload metadata",
            StatusCode::INTERNAL_SERVER_ERROR,
            log::Level::Error,
        )
    })?;
    let saved = DbState::into_inner(saved_state);

    Ok(HttpResponse::Created().json(UploadProjectZipResponse {
        upload_id: saved.upload_id,
        upload_count: saved.upload_count,
        uploads_remaining: (project.max_student_uploads - saved.upload_count).max(0),
    }))
}
