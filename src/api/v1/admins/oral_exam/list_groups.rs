use crate::app_data::AppData;
use crate::common::json_error::{error_with_log_id, JsonError};
use crate::database::repositories::{groups_repository, oral_exam_repository, projects_repository};
use crate::jwt::get_user::LoggedUser;
use actix_web::http::StatusCode;
use actix_web::web::{Data, Path, Query};
use actix_web::{HttpMessage, HttpRequest, HttpResponse};
use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};
use welds::state::DbState;

#[derive(Debug, Deserialize, IntoParams)]
pub(crate) struct GroupSearchQuery {
    pub search: Option<String>,
}

#[derive(Debug, Serialize, ToSchema)]
pub(crate) struct OralExamGroupSummary {
    pub group_id: i32,
    pub name: String,
    pub member_count: i32,
    pub completed_count: i32,
    pub total_members: i32,
}

#[derive(Debug, Serialize, ToSchema)]
pub(crate) struct OralExamGroupListResponse {
    pub project_id: i32,
    pub oral_exam_enabled: bool,
    pub groups: Vec<OralExamGroupSummary>,
}

#[utoipa::path(
    get,
    path = "/v1/admins/oral-exam/projects/{project_id}/groups",
    params(GroupSearchQuery),
    responses(
        (status = 200, description = "Groups listed alphabetically", body = OralExamGroupListResponse),
        (status = 401, description = "Authentication required", body = JsonError),
        (status = 404, description = "Project not found", body = JsonError),
        (status = 500, description = "Internal server error", body = JsonError)
    ),
    security(("AdminAuth" = [])),
    tag = "Admin Oral Exam",
)]
#[actix_web_grants::protect(any("ROLE_ADMIN_ROOT", "ROLE_ADMIN_PROFESSOR"))]
pub(super) async fn list_oral_exam_groups(
    req: HttpRequest, path: Path<i32>, query: Query<GroupSearchQuery>, data: Data<AppData>,
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

    let project_id = path.into_inner();

    let project = projects_repository::get_by_id(&data.db, project_id)
        .await
        .map_err(|e| {
            error_with_log_id(
                format!("unable to fetch project {}: {}", project_id, e),
                "Database error",
                StatusCode::INTERNAL_SERVER_ERROR,
                log::Level::Error,
            )
        })?
        .ok_or_else(|| {
            error_with_log_id(
                format!("project {} not found", project_id),
                "Project not found",
                StatusCode::NOT_FOUND,
                log::Level::Warn,
            )
        })?;

    let oral_exam_enabled = project.oral_exam_enabled;

    let mut groups = groups_repository::get_by_project_id(&data.db, project_id)
        .await
        .map_err(|e| {
            error_with_log_id(
                format!("unable to fetch groups for project {}: {}", project_id, e),
                "Database error",
                StatusCode::INTERNAL_SERVER_ERROR,
                log::Level::Error,
            )
        })?;

    // Apply search filter (case-insensitive)
    if let Some(ref search) = query.search {
        let search_lower = search.to_lowercase();
        groups.retain(|g| g.name.to_lowercase().contains(&search_lower));
    }

    // Sort alphabetically by name
    groups.sort_by(|a, b| a.name.cmp(&b.name));

    let completions = oral_exam_repository::get_completions_for_project(&data.db, project_id)
        .await
        .map_err(|e| {
            error_with_log_id(
                format!(
                    "unable to fetch completions for project {}: {}",
                    project_id, e
                ),
                "Database error",
                StatusCode::INTERNAL_SERVER_ERROR,
                log::Level::Error,
            )
        })?;

    let completed_student_ids: std::collections::HashSet<i32> =
        completions.iter().map(|c| c.student_id).collect();

    let mut summaries = Vec::new();
    for group_state in groups {
        let group = DbState::into_inner(group_state);

        let members = groups_repository::get_group_members(&data.db, group.group_id)
            .await
            .map_err(|e| {
                error_with_log_id(
                    format!(
                        "unable to fetch members for group {}: {}",
                        group.group_id, e
                    ),
                    "Database error",
                    StatusCode::INTERNAL_SERVER_ERROR,
                    log::Level::Error,
                )
            })?;

        let total_members = members.len() as i32;
        let completed_count = members
            .iter()
            .filter(|m| completed_student_ids.contains(&m.student_id))
            .count() as i32;

        summaries.push(OralExamGroupSummary {
            group_id: group.group_id,
            name: group.name,
            member_count: total_members,
            completed_count,
            total_members,
        });
    }

    Ok(HttpResponse::Ok().json(OralExamGroupListResponse {
        project_id,
        oral_exam_enabled,
        groups: summaries,
    }))
}
