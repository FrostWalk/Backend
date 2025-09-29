use crate::app_data::AppData;
use crate::common::json_error::{error_with_log_id, JsonError};
use crate::jwt::get_user::LoggedUser;
use crate::models::group::Group;
use crate::models::group_member::GroupMember;
use crate::models::student::Student;
use crate::models::student_role::AvailableStudentRole;
use actix_web::http::StatusCode;
use actix_web::web::{Data, Json, Path};
use actix_web::{HttpMessage, HttpRequest, HttpResponse};
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
pub(crate) struct MemberResponse {
    pub success: bool,
    pub message: String,
    pub member: Option<MemberInfo>,
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
        (status = 200, description = "Member added successfully", body = MemberResponse),
        (status = 400, description = "Invalid request data or business rule violation", body = JsonError),
        (status = 401, description = "Authentication required", body = JsonError),
        (status = 403, description = "Insufficient permissions", body = JsonError),
        (status = 404, description = "Group not found", body = JsonError),
        (status = 500, description = "Internal server error", body = JsonError)
    ),
    security(("UserAuth" = [])),
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
    if !is_group_leader(&data, user.student_id, group_id).await? {
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
    let student_states = match Student::where_col(|s| s.email.equal(&body.email))
        .run(&data.db)
        .await
    {
        Ok(rows) => rows,
        Err(e) => {
            return Err(error_with_log_id(
                format!("unable to find student with email {}: {}", body.email, e),
                "Database error",
                StatusCode::INTERNAL_SERVER_ERROR,
                log::Level::Error,
            ));
        }
    };

    let student = match student_states.into_iter().next() {
        Some(state) => DbState::into_inner(state),
        None => {
            return Ok(HttpResponse::Ok().json(MemberResponse {
                success: false,
                message: format!("Student with email '{}' not found", body.email),
                member: None,
            }));
        }
    };

    // Check if the student is already in a group for this project
    let group = get_group(&data, group_id).await?;
    if is_student_in_project(&data, student.student_id, group.project_id).await? {
        return Ok(HttpResponse::Ok().json(MemberResponse {
            success: false,
            message: "Student is already in a group for this project".to_string(),
            member: None,
        }));
    }

    // Add the student as a group member with Member role
    let mut member_state = DbState::new_uncreated(GroupMember {
        group_member_id: 0,
        group_id,
        student_id: student.student_id,
        student_role_id: AvailableStudentRole::Member as i32,
    });

    match member_state.save(&data.db).await {
        Ok(_) => Ok(HttpResponse::Ok().json(MemberResponse {
            success: true,
            message: format!("Student '{}' added to group successfully", body.email),
            member: Some(MemberInfo {
                student_id: student.student_id,
                email: student.email,
                first_name: student.first_name,
                last_name: student.last_name,
                role: "Member".to_string(),
            }),
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
        (status = 200, description = "Member removed successfully", body = MemberResponse),
        (status = 400, description = "Invalid request data or business rule violation", body = JsonError),
        (status = 401, description = "Authentication required", body = JsonError),
        (status = 403, description = "Insufficient permissions", body = JsonError),
        (status = 404, description = "Group or member not found", body = JsonError),
        (status = 500, description = "Internal server error", body = JsonError)
    ),
    security(("UserAuth" = [])),
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
    if !is_group_leader(&data, user.student_id, group_id).await? {
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
    let member_states = match GroupMember::where_col(|gm| gm.group_id.equal(group_id))
        .run(&data.db)
        .await
    {
        Ok(rows) => rows,
        Err(e) => {
            return Err(error_with_log_id(
                format!("unable to find group member: {}", e),
                "Database error",
                StatusCode::INTERNAL_SERVER_ERROR,
                log::Level::Error,
            ));
        }
    };

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
            return Ok(HttpResponse::Ok().json(MemberResponse {
                success: false,
                message: "Member not found in this group".to_string(),
                member: None,
            }));
        }
    };

    // Don't allow removing the GroupLeader
    if member.student_role_id == AvailableStudentRole::GroupLeader as i32 {
        return Ok(HttpResponse::Ok().json(MemberResponse {
            success: false,
            message: "Cannot remove the group leader".to_string(),
            member: None,
        }));
    }

    // Remove the member using where_col delete with member ID
    match GroupMember::where_col(|gm| gm.group_member_id.equal(member.group_member_id))
        .delete(&data.db)
        .await
    {
        Ok(_) => Ok(HttpResponse::Ok().json(MemberResponse {
            success: true,
            message: "Member removed from group successfully".to_string(),
            member: None,
        })),
        Err(e) => Err(error_with_log_id(
            format!("unable to remove member from group: {}", e),
            "Database error",
            StatusCode::INTERNAL_SERVER_ERROR,
            log::Level::Error,
        )),
    }
}

// Helper functions
async fn is_group_leader(
    data: &AppData, student_id: i32, group_id: i32,
) -> Result<bool, JsonError> {
    let group_member_states = GroupMember::where_col(|gm| gm.group_id.equal(group_id))
        .run(&data.db)
        .await
        .map_err(|e| {
            error_with_log_id(
                format!("unable to verify group leadership: {}", e),
                "Database error",
                StatusCode::INTERNAL_SERVER_ERROR,
                log::Level::Error,
            )
        })?;

    let mut is_group_leader = false;
    for gm in group_member_states {
        let group_member = DbState::into_inner(gm);
        if group_member.student_id == student_id
            && group_member.student_role_id == AvailableStudentRole::GroupLeader as i32
        {
            is_group_leader = true;
            break;
        }
    }

    Ok(is_group_leader)
}

async fn get_group(data: &AppData, group_id: i32) -> Result<Group, JsonError> {
    let group_states = Group::where_col(|g| g.group_id.equal(group_id))
        .run(&data.db)
        .await
        .map_err(|e| {
            error_with_log_id(
                format!("unable to fetch group {}: {}", group_id, e),
                "Database error",
                StatusCode::INTERNAL_SERVER_ERROR,
                log::Level::Error,
            )
        })?;

    match group_states.into_iter().next() {
        Some(state) => Ok(DbState::into_inner(state)),
        None => Err(error_with_log_id(
            format!("group {} not found", group_id),
            "Group not found",
            StatusCode::NOT_FOUND,
            log::Level::Warn,
        )),
    }
}

async fn is_student_in_project(
    data: &AppData, student_id: i32, project_id: i32,
) -> Result<bool, JsonError> {
    let existing_membership = GroupMember::where_col(|gm| gm.student_id.equal(student_id))
        .run(&data.db)
        .await
        .map_err(|e| {
            error_with_log_id(
                format!(
                    "unable to check existing membership for student {}: {}",
                    student_id, e
                ),
                "Database error",
                StatusCode::INTERNAL_SERVER_ERROR,
                log::Level::Error,
            )
        })?;

    for membership in existing_membership {
        let membership_data = DbState::into_inner(membership);
        let group_states = Group::where_col(|g| g.group_id.equal(membership_data.group_id))
            .run(&data.db)
            .await
            .unwrap_or_default();

        if let Some(group_state) = group_states.into_iter().next() {
            let group = DbState::into_inner(group_state);
            if group.project_id == project_id {
                return Ok(true);
            }
        }
    }

    Ok(false)
}
