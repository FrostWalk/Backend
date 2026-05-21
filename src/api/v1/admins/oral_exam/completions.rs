use crate::app_data::AppData;
use crate::common::json_error::{error_with_log_id, JsonError};
use crate::database::repositories::{groups_repository, oral_exam_repository};
use crate::jwt::get_user::LoggedUser;
use actix_web::http::StatusCode;
use actix_web::web::{Data, Json, Path};
use actix_web::{HttpMessage, HttpRequest, HttpResponse};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

// ── Single student completion ──────────────────────────────────────────────

#[derive(Debug, Deserialize, ToSchema)]
pub(crate) struct SetCompletionRequest {
    pub completed: bool,
}

#[derive(Debug, Serialize, ToSchema)]
pub(crate) struct CompletionResponse {
    pub student_id: i32,
    pub project_id: i32,
    pub completed: bool,
    pub completed_at: Option<DateTime<Utc>>,
}

#[utoipa::path(
    put,
    path = "/v1/admins/oral-exam/projects/{project_id}/students/{student_id}/completion",
    request_body = SetCompletionRequest,
    responses(
        (status = 200, description = "Completion status updated", body = CompletionResponse),
        (status = 401, description = "Authentication required", body = JsonError),
        (status = 500, description = "Internal server error", body = JsonError)
    ),
    security(("AdminAuth" = [])),
    tag = "Admin Oral Exam",
)]
#[actix_web_grants::protect(any("ROLE_ADMIN_ROOT", "ROLE_ADMIN_PROFESSOR"))]
pub(super) async fn set_student_completion(
    req: HttpRequest, path: Path<(i32, i32)>, body: Json<SetCompletionRequest>, data: Data<AppData>,
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

    if body.completed {
        let completion = oral_exam_repository::mark_completed(
            &data.db,
            student_id,
            project_id,
            admin.admin_id,
            Utc::now(),
        )
        .await
        .map_err(|e| {
            error_with_log_id(
                format!(
                    "unable to mark student {} project {} complete: {}",
                    student_id, project_id, e
                ),
                "Database error",
                StatusCode::INTERNAL_SERVER_ERROR,
                log::Level::Error,
            )
        })?;

        Ok(HttpResponse::Ok().json(CompletionResponse {
            student_id,
            project_id,
            completed: true,
            completed_at: Some(completion.completed_at),
        }))
    } else {
        oral_exam_repository::mark_incomplete(&data.db, student_id, project_id)
            .await
            .map_err(|e| {
                error_with_log_id(
                    format!(
                        "unable to mark student {} project {} incomplete: {}",
                        student_id, project_id, e
                    ),
                    "Database error",
                    StatusCode::INTERNAL_SERVER_ERROR,
                    log::Level::Error,
                )
            })?;

        Ok(HttpResponse::Ok().json(CompletionResponse {
            student_id,
            project_id,
            completed: false,
            completed_at: None,
        }))
    }
}

// ── Bulk group completion ──────────────────────────────────────────────────

#[derive(Debug, Deserialize, ToSchema)]
pub(crate) struct BulkCompletionRequest {
    /// Student IDs to mark. If `completed` is true, all listed students are marked done.
    /// If false, all listed students are marked incomplete.
    pub student_ids: Vec<i32>,
    pub completed: bool,
}

#[derive(Debug, Serialize, ToSchema)]
pub(crate) struct BulkCompletionResponse {
    pub project_id: i32,
    pub group_id: i32,
    pub results: Vec<CompletionResponse>,
}

#[utoipa::path(
    post,
    path = "/v1/admins/oral-exam/projects/{project_id}/groups/{group_id}/completions",
    request_body = BulkCompletionRequest,
    responses(
        (status = 200, description = "Bulk completion updated", body = BulkCompletionResponse),
        (status = 401, description = "Authentication required", body = JsonError),
        (status = 404, description = "Group not found", body = JsonError),
        (status = 500, description = "Internal server error", body = JsonError)
    ),
    security(("AdminAuth" = [])),
    tag = "Admin Oral Exam",
)]
#[actix_web_grants::protect(any("ROLE_ADMIN_ROOT", "ROLE_ADMIN_PROFESSOR"))]
pub(super) async fn bulk_set_group_completions(
    req: HttpRequest, path: Path<(i32, i32)>, body: Json<BulkCompletionRequest>,
    data: Data<AppData>,
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

    let (project_id, group_id) = path.into_inner();

    // Verify group exists and belongs to project
    let group = groups_repository::get_by_id(&data.db, group_id)
        .await
        .map_err(|e| {
            error_with_log_id(
                format!("unable to fetch group {}: {}", group_id, e),
                "Database error",
                StatusCode::INTERNAL_SERVER_ERROR,
                log::Level::Error,
            )
        })?
        .ok_or_else(|| {
            error_with_log_id(
                format!("group {} not found", group_id),
                "Group not found",
                StatusCode::NOT_FOUND,
                log::Level::Warn,
            )
        })?;

    if group.project_id != project_id {
        return Err(error_with_log_id(
            format!(
                "group {} does not belong to project {}",
                group_id, project_id
            ),
            "Group not found",
            StatusCode::NOT_FOUND,
            log::Level::Warn,
        ));
    }

    // Validate all student_ids are members of the group
    let group_members = groups_repository::get_group_members(&data.db, group_id)
        .await
        .map_err(|e| {
            error_with_log_id(
                format!("unable to fetch members for group {}: {}", group_id, e),
                "Database error",
                StatusCode::INTERNAL_SERVER_ERROR,
                log::Level::Error,
            )
        })?;

    let member_ids: std::collections::HashSet<i32> =
        group_members.iter().map(|m| m.student_id).collect();

    for sid in &body.student_ids {
        if !member_ids.contains(sid) {
            return Err(error_with_log_id(
                format!("student {} is not a member of group {}", sid, group_id),
                "Student not in group",
                StatusCode::BAD_REQUEST,
                log::Level::Warn,
            ));
        }
    }

    let now = Utc::now();
    let mut results = Vec::new();

    for &sid in &body.student_ids {
        if body.completed {
            let completion = oral_exam_repository::mark_completed(
                &data.db,
                sid,
                project_id,
                admin.admin_id,
                now,
            )
            .await
            .map_err(|e| {
                error_with_log_id(
                    format!("unable to mark student {} complete: {}", sid, e),
                    "Database error",
                    StatusCode::INTERNAL_SERVER_ERROR,
                    log::Level::Error,
                )
            })?;
            results.push(CompletionResponse {
                student_id: sid,
                project_id,
                completed: true,
                completed_at: Some(completion.completed_at),
            });
        } else {
            oral_exam_repository::mark_incomplete(&data.db, sid, project_id)
                .await
                .map_err(|e| {
                    error_with_log_id(
                        format!("unable to mark student {} incomplete: {}", sid, e),
                        "Database error",
                        StatusCode::INTERNAL_SERVER_ERROR,
                        log::Level::Error,
                    )
                })?;
            results.push(CompletionResponse {
                student_id: sid,
                project_id,
                completed: false,
                completed_at: None,
            });
        }
    }

    Ok(HttpResponse::Ok().json(BulkCompletionResponse {
        project_id,
        group_id,
        results,
    }))
}
