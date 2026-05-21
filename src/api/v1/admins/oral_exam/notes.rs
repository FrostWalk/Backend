use crate::app_data::AppData;
use crate::common::json_error::{error_with_log_id, JsonError};
use crate::database::repositories::oral_exam_repository;
use crate::jwt::get_user::LoggedUser;
use actix_web::http::StatusCode;
use actix_web::web::{Data, Json, Path};
use actix_web::{HttpMessage, HttpRequest, HttpResponse};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Deserialize, ToSchema)]
pub(crate) struct UpsertNoteRequest {
    pub text: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub(crate) struct NoteResponse {
    pub student_id: i32,
    pub project_id: i32,
    pub text: String,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

#[utoipa::path(
    put,
    path = "/v1/admins/oral-exam/projects/{project_id}/students/{student_id}/note",
    request_body = UpsertNoteRequest,
    responses(
        (status = 200, description = "Note saved", body = NoteResponse),
        (status = 401, description = "Authentication required", body = JsonError),
        (status = 500, description = "Internal server error", body = JsonError)
    ),
    security(("AdminAuth" = [])),
    tag = "Admin Oral Exam",
)]
#[actix_web_grants::protect(any("ROLE_ADMIN_ROOT", "ROLE_ADMIN_PROFESSOR"))]
pub(super) async fn upsert_note(
    req: HttpRequest, path: Path<(i32, i32)>, body: Json<UpsertNoteRequest>, data: Data<AppData>,
) -> Result<HttpResponse, JsonError> {
    let admin = match req.extensions().get_admin() {
        Ok(admin) => admin,
        Err(_) => {
            return Err(error_with_log_id(
                "entered a protected route without an admin loaded in the request",
                "Authentication error",
                StatusCode::INTERNAL_SERVER_ERROR,
                log::Level::Error,
            ));
        }
    };

    let (project_id, student_id) = path.into_inner();

    let note = oral_exam_repository::upsert_note(
        &data.db,
        student_id,
        project_id,
        body.text.clone(),
        admin.admin_id,
        Utc::now(),
    )
    .await
    .map_err(|e| {
        error_with_log_id(
            format!(
                "unable to upsert note for student {} project {}: {}",
                student_id, project_id, e
            ),
            "Database error",
            StatusCode::INTERNAL_SERVER_ERROR,
            log::Level::Error,
        )
    })?;

    Ok(HttpResponse::Ok().json(NoteResponse {
        student_id,
        project_id,
        text: note.note_text.clone(),
        updated_at: note.updated_at,
    }))
}

#[utoipa::path(
    delete,
    path = "/v1/admins/oral-exam/projects/{project_id}/students/{student_id}/note",
    responses(
        (status = 204, description = "Note deleted"),
        (status = 401, description = "Authentication required", body = JsonError),
        (status = 404, description = "Note not found", body = JsonError),
        (status = 500, description = "Internal server error", body = JsonError)
    ),
    security(("AdminAuth" = [])),
    tag = "Admin Oral Exam",
)]
#[actix_web_grants::protect(any("ROLE_ADMIN_ROOT", "ROLE_ADMIN_PROFESSOR"))]
pub(super) async fn delete_note(
    req: HttpRequest, path: Path<(i32, i32)>, data: Data<AppData>,
) -> Result<HttpResponse, JsonError> {
    let _admin = match req.extensions().get_admin() {
        Ok(admin) => admin,
        Err(_) => {
            return Err(error_with_log_id(
                "entered a protected route without an admin loaded in the request",
                "Authentication error",
                StatusCode::INTERNAL_SERVER_ERROR,
                log::Level::Error,
            ));
        }
    };

    let (project_id, student_id) = path.into_inner();

    let deleted = oral_exam_repository::delete_note(&data.db, student_id, project_id)
        .await
        .map_err(|e| {
            error_with_log_id(
                format!(
                    "unable to delete note for student {} project {}: {}",
                    student_id, project_id, e
                ),
                "Database error",
                StatusCode::INTERNAL_SERVER_ERROR,
                log::Level::Error,
            )
        })?;

    if !deleted {
        return Err(error_with_log_id(
            format!(
                "note for student {} project {} not found",
                student_id, project_id
            ),
            "Note not found",
            StatusCode::NOT_FOUND,
            log::Level::Warn,
        ));
    }

    Ok(HttpResponse::NoContent().finish())
}
