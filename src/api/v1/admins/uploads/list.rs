use crate::app_data::AppData;
use crate::common::json_error::{error_with_log_id, JsonError, ToJsonError};
use crate::database::repositories::{projects_repository, student_uploads_repository};
use crate::jwt::get_user::LoggedUser;
use actix_web::http::StatusCode;
use actix_web::web::{Data, Path};
use actix_web::{HttpMessage, HttpRequest, HttpResponse};
use chrono::{DateTime, Utc};
use serde::Serialize;
use utoipa::ToSchema;

#[derive(Debug, Serialize, ToSchema)]
pub(crate) struct ProjectUploadItem {
    pub student_id: i32,
    pub first_name: String,
    pub last_name: String,
    pub upload_count: i32,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Serialize, ToSchema)]
pub(crate) struct ListProjectUploadsResponse {
    pub project_id: i32,
    pub uploads: Vec<ProjectUploadItem>,
}

#[utoipa::path(
    get,
    path = "/v1/admins/projects/{project_id}/uploads",
    params(
        ("project_id" = i32, Path, description = "Project id")
    ),
    responses(
        (status = 200, description = "Project uploads list", body = ListProjectUploadsResponse),
        (status = 401, description = "Authentication required", body = JsonError),
        (status = 404, description = "Project not found", body = JsonError),
        (status = 500, description = "Internal server error", body = JsonError),
    ),
    security(("AdminAuth" = [])),
    tag = "Student Uploads",
)]
#[actix_web_grants::protect(any("ROLE_ADMIN_ROOT", "ROLE_ADMIN_PROFESSOR"))]
pub(in crate::api::v1) async fn list_project_uploads_handler(
    req: HttpRequest, path: Path<i32>, data: Data<AppData>,
) -> Result<HttpResponse, JsonError> {
    let _admin = req.extensions().get_admin().map_err(|_| {
        error_with_log_id(
            "entered protected upload list route without loaded admin",
            "Authentication error",
            StatusCode::INTERNAL_SERVER_ERROR,
            log::Level::Error,
        )
    })?;

    let project_id = path.into_inner();
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

    let uploads = student_uploads_repository::get_all_by_project(&data.db, project_id)
        .await
        .map_err(|e| {
            error_with_log_id(
                format!("failed loading uploads for project {}: {}", project_id, e),
                "Database error",
                StatusCode::INTERNAL_SERVER_ERROR,
                log::Level::Error,
            )
        })?;

    let upload_items = uploads
        .into_iter()
        .map(|(upload, student)| ProjectUploadItem {
            student_id: student.student_id,
            first_name: student.first_name,
            last_name: student.last_name,
            upload_count: upload.upload_count,
            timestamp: upload.timestamp,
        })
        .collect();

    Ok(HttpResponse::Ok().json(ListProjectUploadsResponse {
        project_id,
        uploads: upload_items,
    }))
}
