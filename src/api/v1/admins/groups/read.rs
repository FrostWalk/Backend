use crate::app_data::AppData;
use crate::common::json_error::{error_with_log_id, JsonError};
use crate::database::repositories::{
    group_deliverable_selections_repository, group_deliverables_repository, groups_repository,
    projects_repository, students_repository,
};
use crate::jwt::get_user::LoggedUser;
use actix_web::http::StatusCode;
use actix_web::web::{Data, Path};
use actix_web::{HttpMessage, HttpRequest, HttpResponse};
use chrono::Utc;
use serde::Serialize;
use utoipa::ToSchema;
use welds::state::DbState;

#[derive(Debug, Serialize, ToSchema)]
pub(crate) struct ProjectGroupsResponse {
    pub groups: Vec<GroupInfo>,
}

#[derive(Debug, Serialize, ToSchema)]
pub(crate) struct GroupInfo {
    pub group_id: i32,
    pub name: String,
    pub member_count: i32,
    pub group_leader: GroupLeaderInfo,
    pub deliverable_selected: Option<DeliverableInfo>,
    pub time_expired: bool,
}

#[derive(Debug, Serialize, ToSchema)]
pub(crate) struct GroupLeaderInfo {
    pub student_id: i32,
    pub name: String,
    pub email: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub(crate) struct DeliverableInfo {
    pub group_deliverable_id: i32,
    pub name: String,
}

#[utoipa::path(
    get,
    path = "/v1/admins/groups/projects/{project_id}",
    responses(
        (status = 200, description = "Project groups list", body = ProjectGroupsResponse),
        (status = 401, description = "Authentication required", body = JsonError),
        (status = 404, description = "Project not found", body = JsonError),
        (status = 500, description = "Internal server error", body = JsonError)
    ),
    security(("AdminAuth" = [])),
    tag = "Admin Groups management",
)]
/// List all groups in a project with their members and deliverable selections
///
/// This endpoint allows admins to view all groups in a project with member counts,
/// group leaders, and their chosen deliverables. Includes time_expired field for
/// groups that haven't selected a deliverable by the deadline.
#[actix_web_grants::protect(any(
    "ROLE_ADMIN_ROOT",
    "ROLE_ADMIN_PROFESSOR",
    "ROLE_ADMIN_COORDINATOR"
))]
pub(super) async fn get_project_groups(
    req: HttpRequest, path: Path<i32>, data: Data<AppData>,
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

    // Verify the project exists
    let project_state = projects_repository::get_by_id(&data.db, project_id)
        .await
        .map_err(|e| {
            error_with_log_id(
                format!("unable to fetch project: {}", e),
                "Database error",
                StatusCode::INTERNAL_SERVER_ERROR,
                log::Level::Error,
            )
        })?;

    let project = match project_state {
        Some(state) => DbState::into_inner(state),
        None => {
            return Err(error_with_log_id(
                format!("project with id {} not found", project_id),
                "Project not found",
                StatusCode::NOT_FOUND,
                log::Level::Warn,
            ));
        }
    };

    // Get all groups for this project
    let groups = groups_repository::get_by_project_id(&data.db, project_id)
        .await
        .map_err(|e| {
            error_with_log_id(
                format!("unable to fetch groups for project {}: {}", project_id, e),
                "Database error",
                StatusCode::INTERNAL_SERVER_ERROR,
                log::Level::Error,
            )
        })?;

    let mut group_infos = Vec::new();

    for group_state in groups {
        let group = DbState::into_inner(group_state);

        // Get group members
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

        let member_count = members.len() as i32;

        // Find the group leader
        let mut group_leader = None;
        for member_state in members {
            let member = DbState::into_inner(member_state);
            if member.student_role_id == 1 {
                // Group Leader role
                let student_state = students_repository::get_by_id(&data.db, member.student_id)
                    .await
                    .map_err(|e| {
                        error_with_log_id(
                            format!("unable to fetch student details: {}", e),
                            "Database error",
                            StatusCode::INTERNAL_SERVER_ERROR,
                            log::Level::Error,
                        )
                    })?;

                if let Some(student_state) = student_state {
                    let student = DbState::into_inner(student_state);
                    group_leader = Some(GroupLeaderInfo {
                        student_id: student.student_id,
                        name: format!("{} {}", student.first_name, student.last_name),
                        email: student.email,
                    });
                    break;
                }
            }
        }

        let group_leader = group_leader.unwrap_or(GroupLeaderInfo {
            student_id: 0,
            name: "Unknown".to_string(),
            email: "unknown@example.com".to_string(),
        });

        // Get the group's deliverable selection
        let deliverable_selection =
            group_deliverable_selections_repository::get_by_group_id(&data.db, group.group_id)
                .await
                .map_err(|e| {
                    error_with_log_id(
                        format!(
                            "unable to fetch deliverable selection for group {}: {}",
                            group.group_id, e
                        ),
                        "Database error",
                        StatusCode::INTERNAL_SERVER_ERROR,
                        log::Level::Error,
                    )
                })?;

        let deliverable_selected = if let Some(selection_state) = deliverable_selection {
            let selection = DbState::into_inner(selection_state);
            // Get the deliverable name
            let deliverable_state =
                group_deliverables_repository::get_by_id(&data.db, selection.group_deliverable_id)
                    .await
                    .map_err(|e| {
                        error_with_log_id(
                            format!(
                                "unable to fetch deliverable {}: {}",
                                selection.group_deliverable_id, e
                            ),
                            "Database error",
                            StatusCode::INTERNAL_SERVER_ERROR,
                            log::Level::Error,
                        )
                    })?;

            if let Some(deliverable_state) = deliverable_state {
                let deliverable = DbState::into_inner(deliverable_state);
                Some(DeliverableInfo {
                    group_deliverable_id: selection.group_deliverable_id,
                    name: deliverable.name,
                })
            } else {
                Some(DeliverableInfo {
                    group_deliverable_id: selection.group_deliverable_id,
                    name: format!("Unknown Deliverable {}", selection.group_deliverable_id),
                })
            }
        } else {
            None
        };

        // Check if time has expired for deliverable selection
        let time_expired = if let Some(deadline) = project.deliverable_selection_deadline {
            deliverable_selected.is_none() && Utc::now() > deadline
        } else {
            false
        };

        group_infos.push(GroupInfo {
            group_id: group.group_id,
            name: group.name,
            member_count,
            group_leader,
            deliverable_selected,
            time_expired,
        });
    }

    Ok(HttpResponse::Ok().json(ProjectGroupsResponse {
        groups: group_infos,
    }))
}
