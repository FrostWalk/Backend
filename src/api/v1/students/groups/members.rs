use crate::app_data::AppData;
use crate::common::json_error::{error_with_log_id, JsonError};
use crate::database::repositories::{
    groups_repository, projects_repository, student_deliverable_selections_repository,
    students_repository,
};
use crate::jwt::get_user::LoggedUser;
use crate::models::group_member::GroupMember;
use crate::models::student_role::AvailableStudentRole;
use actix_web::http::StatusCode;
use actix_web::web::{Data, Json, Path};
use actix_web::{HttpMessage, HttpRequest, HttpResponse};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use welds::state::DbState;

#[derive(Debug, Deserialize, ToSchema)]
pub(crate) struct AddMemberRequest {
    pub email: String,
}

#[derive(Debug, Deserialize, ToSchema)]
pub(crate) struct RemoveMemberRequest {
    pub student_id: i32,
}

#[derive(Debug, Serialize, ToSchema)]
pub(crate) struct MemberInfo {
    pub student_id: i32,
    pub email: String,
    pub first_name: String,
    pub last_name: String,
    pub role: String,
}

#[utoipa::path(
    post,
    path = "/v1/students/groups/{group_id}/members",
    request_body = AddMemberRequest,
    responses(
        (status = 200, description = "Member added successfully", body = MemberInfo),
        (status = 400, description = "Student email not confirmed or group at maximum capacity", body = JsonError),
        (status = 401, description = "Authentication required", body = JsonError),
        (status = 403, description = "Insufficient permissions", body = JsonError),
        (status = 404, description = "Group or student not found", body = JsonError),
        (status = 409, description = "Student is already in a group for this project", body = JsonError),
        (status = 500, description = "Internal server error", body = JsonError)
    ),
    security(("StudentAuth" = [])),
    tag = "Groups management",
)]
/// Add a member to a group
///
/// This endpoint allows GroupLeaders to add new members to their group.
pub(super) async fn add_member(
    req: HttpRequest, data: Data<AppData>, path: Path<i32>, body: Json<AddMemberRequest>,
) -> Result<HttpResponse, JsonError> {
    let user = match req.extensions().get_student() {
        Ok(user) => user,
        Err(_) => {
            return Err(error_with_log_id(
                "entered a protected route without a user loaded in the request",
                "Authentication error",
                StatusCode::INTERNAL_SERVER_ERROR,
                log::Level::Error,
            ));
        }
    };

    let group_id = path.into_inner();

    // Verify the user is a GroupLeader of this group
    let is_leader = groups_repository::is_group_leader(&data.db, user.student_id, group_id)
        .await
        .map_err(|e| {
            error_with_log_id(
                format!("unable to verify group leadership: {}", e),
                "Database error",
                StatusCode::INTERNAL_SERVER_ERROR,
                log::Level::Error,
            )
        })?;

    if !is_leader {
        return Err(error_with_log_id(
            format!(
                "user {} is not a GroupLeader of group {}",
                user.student_id, group_id
            ),
            "Insufficient permissions",
            StatusCode::FORBIDDEN,
            log::Level::Warn,
        ));
    }

    // Find the student by email
    let student_state = students_repository::get_by_email(&data.db, &body.email)
        .await
        .map_err(|e| {
            error_with_log_id(
                format!("unable to find student with email {}: {}", body.email, e),
                "Database error",
                StatusCode::INTERNAL_SERVER_ERROR,
                log::Level::Error,
            )
        })?;

    let student = match student_state {
        Some(state) => DbState::into_inner(state),
        None => {
            return Err(error_with_log_id(
                format!("student with email '{}' not found", body.email),
                format!("Student with email '{}' not found", body.email),
                StatusCode::NOT_FOUND,
                log::Level::Info,
            ));
        }
    };

    // Verify the student has confirmed their email
    if student.is_pending {
        return Err(error_with_log_id(
            format!(
                "student {} has not confirmed their email",
                student.student_id
            ),
            "Student must confirm their email before joining a group",
            StatusCode::BAD_REQUEST,
            log::Level::Info,
        ));
    }

    // Get the group and check if the student is already in a group for this project
    let group_state = groups_repository::get_by_id(&data.db, group_id)
        .await
        .map_err(|e| {
            error_with_log_id(
                format!("unable to fetch group {}: {}", group_id, e),
                "Database error",
                StatusCode::INTERNAL_SERVER_ERROR,
                log::Level::Error,
            )
        })?;

    let group = match group_state {
        Some(state) => DbState::into_inner(state),
        None => {
            return Err(error_with_log_id(
                format!("group {} not found", group_id),
                "Group not found",
                StatusCode::NOT_FOUND,
                log::Level::Warn,
            ));
        }
    };

    let in_project =
        groups_repository::is_student_in_project(&data.db, student.student_id, group.project_id)
            .await
            .map_err(|e| {
                error_with_log_id(
                    format!(
                        "unable to check existing membership for student {}: {}",
                        student.student_id, e
                    ),
                    "Database error",
                    StatusCode::INTERNAL_SERVER_ERROR,
                    log::Level::Error,
                )
            })?;

    if in_project {
        return Err(error_with_log_id(
            format!(
                "student {} is already in a group for project {}",
                student.student_id, group.project_id
            ),
            "Student is already in a group for this project",
            StatusCode::CONFLICT,
            log::Level::Info,
        ));
    }

    // Check if adding this member would exceed the maximum group size
    let project_state = projects_repository::get_by_id(&data.db, group.project_id)
        .await
        .map_err(|e| {
            error_with_log_id(
                format!("unable to fetch project {}: {}", group.project_id, e),
                "Database error",
                StatusCode::INTERNAL_SERVER_ERROR,
                log::Level::Error,
            )
        })?;

    let project = match project_state {
        Some(state) => DbState::into_inner(state),
        None => {
            return Err(error_with_log_id(
                format!("project {} not found", group.project_id),
                "Project not found",
                StatusCode::NOT_FOUND,
                log::Level::Warn,
            ));
        }
    };

    let current_member_count = groups_repository::count_members(&data.db, group_id)
        .await
        .map_err(|e| {
            error_with_log_id(
                format!("unable to count members for group {}: {}", group_id, e),
                "Database error",
                StatusCode::INTERNAL_SERVER_ERROR,
                log::Level::Error,
            )
        })?;

    if current_member_count >= project.max_group_size {
        return Err(error_with_log_id(
            format!(
                "group {} has reached maximum size of {} members",
                group_id, project.max_group_size
            ),
            format!(
                "Group has reached the maximum size of {} members for this project",
                project.max_group_size
            ),
            StatusCode::BAD_REQUEST,
            log::Level::Info,
        ));
    }

    // Add the student as a group member with Member role
    let mut member_state = DbState::new_uncreated(GroupMember {
        group_member_id: 0,
        group_id,
        student_id: student.student_id,
        student_role_id: AvailableStudentRole::Member as i32,
        joined_at: Utc::now(),
    });

    match member_state.save(&data.db).await {
        Ok(_) => Ok(HttpResponse::Ok().json(MemberInfo {
            student_id: student.student_id,
            email: student.email,
            first_name: student.first_name,
            last_name: student.last_name,
            role: "Member".to_string(),
        })),
        Err(e) => Err(error_with_log_id(
            format!(
                "unable to add student {} to group: {}",
                student.student_id, e
            ),
            "Database error",
            StatusCode::INTERNAL_SERVER_ERROR,
            log::Level::Error,
        )),
    }
}

#[utoipa::path(
    delete,
    path = "/v1/students/groups/{group_id}/members",
    request_body = RemoveMemberRequest,
    responses(
        (status = 204, description = "Member removed successfully"),
        (status = 400, description = "Cannot remove the group leader", body = JsonError),
        (status = 401, description = "Authentication required", body = JsonError),
        (status = 403, description = "Insufficient permissions", body = JsonError),
        (status = 404, description = "Group or member not found", body = JsonError),
        (status = 500, description = "Internal server error", body = JsonError)
    ),
    security(("StudentAuth" = [])),
    tag = "Groups management",
)]
/// Remove a member from a group
///
/// This endpoint allows GroupLeaders to remove members from their group.
pub(super) async fn remove_member(
    req: HttpRequest, data: Data<AppData>, path: Path<i32>, body: Json<RemoveMemberRequest>,
) -> Result<HttpResponse, JsonError> {
    let user = match req.extensions().get_student() {
        Ok(user) => user,
        Err(_) => {
            return Err(error_with_log_id(
                "entered a protected route without a user loaded in the request",
                "Authentication error",
                StatusCode::INTERNAL_SERVER_ERROR,
                log::Level::Error,
            ));
        }
    };

    let group_id = path.into_inner();

    // Verify the user is a GroupLeader of this group
    let is_leader = groups_repository::is_group_leader(&data.db, user.student_id, group_id)
        .await
        .map_err(|e| {
            error_with_log_id(
                format!("unable to verify group leadership: {}", e),
                "Database error",
                StatusCode::INTERNAL_SERVER_ERROR,
                log::Level::Error,
            )
        })?;

    if !is_leader {
        return Err(error_with_log_id(
            format!(
                "user {} is not a GroupLeader of group {}",
                user.student_id, group_id
            ),
            "Insufficient permissions",
            StatusCode::FORBIDDEN,
            log::Level::Warn,
        ));
    }

    // Find the group member
    let member_states = groups_repository::get_members(&data.db, group_id)
        .await
        .map_err(|e| {
            error_with_log_id(
                format!("unable to find group member: {}", e),
                "Database error",
                StatusCode::INTERNAL_SERVER_ERROR,
                log::Level::Error,
            )
        })?;

    // Filter by student_id
    let mut found_member = None;
    for member_state in member_states {
        let member_data = DbState::into_inner(member_state);
        if member_data.student_id == body.student_id {
            found_member = Some(member_data);
            break;
        }
    }

    let member = found_member;

    let member = match member {
        Some(member) => member,
        None => {
            return Err(error_with_log_id(
                format!(
                    "member with student_id {} not found in group {}",
                    body.student_id, group_id
                ),
                "Member not found in this group",
                StatusCode::NOT_FOUND,
                log::Level::Info,
            ));
        }
    };

    // Don't allow removing the GroupLeader
    if member.student_role_id == AvailableStudentRole::GroupLeader as i32 {
        return Err(error_with_log_id(
            format!(
                "attempt to remove group leader (student_id {}) from group {}",
                member.student_id, group_id
            ),
            "Cannot remove the group leader",
            StatusCode::BAD_REQUEST,
            log::Level::Info,
        ));
    }

    // Get the group to find the project_id for deliverable selection deletion
    let group_state = groups_repository::get_by_id(&data.db, group_id)
        .await
        .map_err(|e| {
            error_with_log_id(
                format!("unable to fetch group {}: {}", group_id, e),
                "Database error",
                StatusCode::INTERNAL_SERVER_ERROR,
                log::Level::Error,
            )
        })?;

    let group = match group_state {
        Some(state) => DbState::into_inner(state),
        None => {
            return Err(error_with_log_id(
                format!("group {} not found", group_id),
                "Group not found",
                StatusCode::NOT_FOUND,
                log::Level::Warn,
            ));
        }
    };

    // Delete the student's deliverable selection for this project (MANDATORY - Q4)
    if let Err(e) = student_deliverable_selections_repository::delete_by_student_and_project(
        &data.db,
        member.student_id,
        group.project_id,
    )
    .await
    {
        // Log the error but don't fail the operation - the member should still be removed
        log::warn!(
            "Failed to delete deliverable selection for student {} in project {}: {}",
            member.student_id,
            group.project_id,
            e
        );
    }

    // Remove the member using where_col delete with member ID
    match GroupMember::where_col(|gm| gm.group_member_id.equal(member.group_member_id))
        .delete(&data.db)
        .await
    {
        Ok(_) => Ok(HttpResponse::NoContent().finish()),
        Err(e) => Err(error_with_log_id(
            format!("unable to remove member from group: {}", e),
            "Database error",
            StatusCode::INTERNAL_SERVER_ERROR,
            log::Level::Error,
        )),
    }
}
