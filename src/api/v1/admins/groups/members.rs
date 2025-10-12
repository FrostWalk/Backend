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
pub(crate) struct TransferLeadershipRequest {
    pub new_leader_student_id: i32,
    pub remove_old_leader: bool,
}

#[derive(Debug, Deserialize, ToSchema)]
pub(crate) struct AdminAddMemberRequest {
    pub student_email: String,
    pub role_id: i32,
}

#[derive(Debug, Serialize, ToSchema)]
pub(crate) struct AdminMemberResponse {
    pub success: bool,
    pub message: String,
    pub member: Option<AdminMemberInfo>,
}

#[derive(Debug, Serialize, ToSchema)]
pub(crate) struct AdminMemberInfo {
    pub student_id: i32,
    pub name: String,
    pub email: String,
    pub role: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub(crate) struct TransferLeadershipResponse {
    pub message: String,
    pub old_leader: Option<LeaderChangeInfo>,
    pub new_leader: LeaderChangeInfo,
}

#[derive(Debug, Serialize, ToSchema)]
pub(crate) struct LeaderChangeInfo {
    pub student_id: i32,
    pub name: String,
    pub status: String,
}

#[utoipa::path(
    delete,
    path = "/v1/admins/groups/{group_id}/members/{student_id}",
    responses(
        (status = 200, description = "Member removed successfully", body = AdminMemberResponse),
        (status = 401, description = "Authentication required", body = JsonError),
        (status = 404, description = "Group or member not found", body = JsonError),
        (status = 500, description = "Internal server error", body = JsonError)
    ),
    security(("AdminAuth" = [])),
    tag = "Admin Groups management",
)]
/// Remove a member from a group (Admin/Coordinator)
///
/// This endpoint allows admins and coordinators to remove any member from a group,
/// including the Group Leader. Must delete the student's deliverable selection when removed.
pub(super) async fn remove_member(
    req: HttpRequest, data: Data<AppData>, path: Path<(i32, i32)>,
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

    let (group_id, student_id) = path.into_inner();

    // Verify the group exists
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

    // Find the group member
    let members = groups_repository::get_group_members(&data.db, group_id)
        .await
        .map_err(|e| {
            error_with_log_id(
                format!("unable to find group members: {}", e),
                "Database error",
                StatusCode::INTERNAL_SERVER_ERROR,
                log::Level::Error,
            )
        })?;

    let mut found_member = None;
    for member_state in members {
        let member_data = DbState::into_inner(member_state);
        if member_data.student_id == student_id {
            found_member = Some(member_data);
            break;
        }
    }

    let member = match found_member {
        Some(member) => member,
        None => {
            return Ok(HttpResponse::Ok().json(AdminMemberResponse {
                success: false,
                message: "Member not found in this group".to_string(),
                member: None,
            }));
        }
    };

    // Get student details for response
    let student_state = students_repository::get_by_id(&data.db, student_id)
        .await
        .map_err(|e| {
            error_with_log_id(
                format!("unable to fetch student details: {}", e),
                "Database error",
                StatusCode::INTERNAL_SERVER_ERROR,
                log::Level::Error,
            )
        })?;

    let student = match student_state {
        Some(state) => DbState::into_inner(state),
        None => {
            return Err(error_with_log_id(
                format!("student {} not found", student_id),
                "Student not found",
                StatusCode::NOT_FOUND,
                log::Level::Warn,
            ));
        }
    };

    // Delete the student's deliverable selection for this project (MANDATORY - Q4)
    if let Err(e) = student_deliverable_selections_repository::delete_by_student_and_project(
        &data.db,
        student_id,
        group.project_id,
    )
    .await
    {
        // Log the error but don't fail the operation - the member should still be removed
        log::warn!(
            "Failed to delete deliverable selection for student {} in project {}: {}",
            student_id,
            group.project_id,
            e
        );
    }

    // Remove the member
    match GroupMember::where_col(|gm| gm.group_member_id.equal(member.group_member_id))
        .delete(&data.db)
        .await
    {
        Ok(_) => {
            let role_name = if member.student_role_id == AvailableStudentRole::GroupLeader as i32 {
                "Group Leader"
            } else {
                "Member"
            };

            Ok(HttpResponse::Ok().json(AdminMemberResponse {
                success: true,
                message: "Member removed successfully from the group".to_string(),
                member: Some(AdminMemberInfo {
                    student_id: student.student_id,
                    name: format!("{} {}", student.first_name, student.last_name),
                    email: student.email,
                    role: role_name.to_string(),
                }),
            }))
        }
        Err(e) => Err(error_with_log_id(
            format!("unable to remove member from group: {}", e),
            "Database error",
            StatusCode::INTERNAL_SERVER_ERROR,
            log::Level::Error,
        )),
    }
}

#[utoipa::path(
    patch,
    path = "/v1/admins/groups/{group_id}/leader",
    request_body = TransferLeadershipRequest,
    responses(
        (status = 200, description = "Group leader updated successfully", body = TransferLeadershipResponse),
        (status = 400, description = "Invalid request data", body = JsonError),
        (status = 401, description = "Authentication required", body = JsonError),
        (status = 404, description = "Group or member not found", body = JsonError),
        (status = 500, description = "Internal server error", body = JsonError)
    ),
    security(("AdminAuth" = [])),
    tag = "Admin Groups management",
)]
/// Transfer group leadership (Admin/Coordinator)
///
/// This endpoint allows admins and coordinators to change the Group Leader of a group.
/// Can optionally remove the old leader or demote them to member.
pub(super) async fn transfer_leadership(
    req: HttpRequest, data: Data<AppData>, group_id: Path<i32>,
    body: Json<TransferLeadershipRequest>,
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

    // Verify the group exists
    let group_state = groups_repository::get_by_id(&data.db, *group_id)
        .await
        .map_err(|e| {
            error_with_log_id(
                format!("unable to fetch group {}: {}", group_id, e),
                "Database error",
                StatusCode::INTERNAL_SERVER_ERROR,
                log::Level::Error,
            )
        })?;

    let _group = match group_state {
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

    // Get all group members
    let members = groups_repository::get_group_members(&data.db, *group_id)
        .await
        .map_err(|e| {
            error_with_log_id(
                format!("unable to fetch group members: {}", e),
                "Database error",
                StatusCode::INTERNAL_SERVER_ERROR,
                log::Level::Error,
            )
        })?;

    // Find current leader and new leader
    let mut current_leader = None;
    let mut new_leader = None;

    for member_state in members {
        let member = DbState::into_inner(member_state);
        if member.student_role_id == AvailableStudentRole::GroupLeader as i32 {
            current_leader = Some(member);
        } else if member.student_id == body.new_leader_student_id {
            new_leader = Some(member);
        }
    }

    let current_leader = match current_leader {
        Some(leader) => leader,
        None => {
            return Err(error_with_log_id(
                "no current group leader found",
                "Group has no leader",
                StatusCode::BAD_REQUEST,
                log::Level::Warn,
            ));
        }
    };

    let new_leader = match new_leader {
        Some(leader) => leader,
        None => {
            return Err(error_with_log_id(
                format!(
                    "student {} is not a member of this group",
                    body.new_leader_student_id
                ),
                "New leader not found in group",
                StatusCode::NOT_FOUND,
                log::Level::Warn,
            ));
        }
    };

    // Get student details for responses
    let current_student_state = students_repository::get_by_id(&data.db, current_leader.student_id)
        .await
        .map_err(|e| {
            error_with_log_id(
                format!("unable to fetch current leader details: {}", e),
                "Database error",
                StatusCode::INTERNAL_SERVER_ERROR,
                log::Level::Error,
            )
        })?;

    let new_student_state = students_repository::get_by_id(&data.db, new_leader.student_id)
        .await
        .map_err(|e| {
            error_with_log_id(
                format!("unable to fetch new leader details: {}", e),
                "Database error",
                StatusCode::INTERNAL_SERVER_ERROR,
                log::Level::Error,
            )
        })?;

    let current_student = match current_student_state {
        Some(state) => DbState::into_inner(state),
        None => {
            return Err(error_with_log_id(
                "current leader student not found",
                "Student not found",
                StatusCode::NOT_FOUND,
                log::Level::Warn,
            ));
        }
    };

    let new_student = match new_student_state {
        Some(state) => DbState::into_inner(state),
        None => {
            return Err(error_with_log_id(
                "new leader student not found",
                "Student not found",
                StatusCode::NOT_FOUND,
                log::Level::Warn,
            ));
        }
    };

    // Update roles in database
    // Demote current leader to member (or remove if requested)
    if body.remove_old_leader {
        // Remove the old leader entirely
        match GroupMember::where_col(|gm| gm.group_member_id.equal(current_leader.group_member_id))
            .delete(&data.db)
            .await
        {
            Ok(_) => {}
            Err(e) => {
                return Err(error_with_log_id(
                    format!("unable to remove old leader: {}", e),
                    "Database error",
                    StatusCode::INTERNAL_SERVER_ERROR,
                    log::Level::Error,
                ));
            }
        }
    } else {
        // Demote to member - we need to get the current leader as DbState first
        let current_leader_states =
            GroupMember::where_col(|gm| gm.group_member_id.equal(current_leader.group_member_id))
                .run(&data.db)
                .await
                .map_err(|e| {
                    error_with_log_id(
                        format!("unable to fetch current leader: {}", e),
                        "Database error",
                        StatusCode::INTERNAL_SERVER_ERROR,
                        log::Level::Error,
                    )
                })?;

        if let Some(mut current_leader_state) = current_leader_states.into_iter().next() {
            current_leader_state.as_mut().student_role_id = AvailableStudentRole::Member as i32;

            match current_leader_state.save(&data.db).await {
                Ok(_) => {}
                Err(e) => {
                    return Err(error_with_log_id(
                        format!("unable to demote old leader: {}", e),
                        "Database error",
                        StatusCode::INTERNAL_SERVER_ERROR,
                        log::Level::Error,
                    ));
                }
            }
        }
    }

    // Promote new leader - we need to get the new leader as DbState first
    let new_leader_states =
        GroupMember::where_col(|gm| gm.group_member_id.equal(new_leader.group_member_id))
            .run(&data.db)
            .await
            .map_err(|e| {
                error_with_log_id(
                    format!("unable to fetch new leader: {}", e),
                    "Database error",
                    StatusCode::INTERNAL_SERVER_ERROR,
                    log::Level::Error,
                )
            })?;

    if let Some(mut new_leader_state) = new_leader_states.into_iter().next() {
        new_leader_state.as_mut().student_role_id = AvailableStudentRole::GroupLeader as i32;

        match new_leader_state.save(&data.db).await {
            Ok(_) => {}
            Err(e) => {
                return Err(error_with_log_id(
                    format!("unable to promote new leader: {}", e),
                    "Database error",
                    StatusCode::INTERNAL_SERVER_ERROR,
                    log::Level::Error,
                ));
            }
        }
    }

    let old_leader_info = if body.remove_old_leader {
        Some(LeaderChangeInfo {
            student_id: current_leader.student_id,
            name: format!(
                "{} {}",
                current_student.first_name, current_student.last_name
            ),
            status: "removed_from_group".to_string(),
        })
    } else {
        Some(LeaderChangeInfo {
            student_id: current_leader.student_id,
            name: format!(
                "{} {}",
                current_student.first_name, current_student.last_name
            ),
            status: "demoted_to_member".to_string(),
        })
    };

    let new_leader_info = LeaderChangeInfo {
        student_id: new_leader.student_id,
        name: format!("{} {}", new_student.first_name, new_student.last_name),
        status: "promoted_to_leader".to_string(),
    };

    Ok(HttpResponse::Ok().json(TransferLeadershipResponse {
        message: "Group leader updated successfully".to_string(),
        old_leader: old_leader_info,
        new_leader: new_leader_info,
    }))
}

#[utoipa::path(
    post,
    path = "/v1/admins/groups/{group_id}/members",
    request_body = AdminAddMemberRequest,
    responses(
        (status = 201, description = "Member added successfully", body = AdminMemberResponse),
        (status = 400, description = "Invalid request data or business rule violation", body = JsonError),
        (status = 401, description = "Authentication required", body = JsonError),
        (status = 404, description = "Group or student not found", body = JsonError),
        (status = 500, description = "Internal server error", body = JsonError)
    ),
    security(("AdminAuth" = [])),
    tag = "Admin Groups management",
)]
/// Add a member to a group (Admin/Coordinator)
///
/// This endpoint allows admins and coordinators to manually add students to groups.
/// Can add students as members or group leaders.
pub(super) async fn add_member(
    req: HttpRequest, data: Data<AppData>, group_id: Path<i32>, body: Json<AdminAddMemberRequest>,
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

    // Verify the group exists
    let group_state = groups_repository::get_by_id(&data.db, *group_id)
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

    // Find the student by email
    let student_state = students_repository::get_by_email(&data.db, &body.student_email)
        .await
        .map_err(|e| {
            error_with_log_id(
                format!(
                    "unable to find student with email {}: {}",
                    body.student_email, e
                ),
                "Database error",
                StatusCode::INTERNAL_SERVER_ERROR,
                log::Level::Error,
            )
        })?;

    let student = match student_state {
        Some(state) => DbState::into_inner(state),
        None => {
            return Err(error_with_log_id(
                format!("student with email '{}' not found", body.student_email),
                "Student not found",
                StatusCode::NOT_FOUND,
                log::Level::Warn,
            ));
        }
    };

    // Verify the student has confirmed their email
    if student.is_pending {
        return Err(error_with_log_id(
            "student must confirm their email before joining a group",
            "Student email not confirmed",
            StatusCode::BAD_REQUEST,
            log::Level::Warn,
        ));
    }

    // Check if student is already in a group for this project
    let in_project =
        groups_repository::is_student_in_project(&data.db, student.student_id, group.project_id)
            .await
            .map_err(|e| {
                error_with_log_id(
                    format!("unable to check existing membership: {}", e),
                    "Database error",
                    StatusCode::INTERNAL_SERVER_ERROR,
                    log::Level::Error,
                )
            })?;

    if in_project {
        return Err(error_with_log_id(
            "student is already in a group for this project",
            "Student already in project group",
            StatusCode::BAD_REQUEST,
            log::Level::Warn,
        ));
    }

    // Get project details for group size validation
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

    // Check group size limit
    let current_member_count = groups_repository::count_members(&data.db, *group_id)
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
                "group has reached the maximum size of {} members",
                project.max_group_size
            ),
            "Group size limit exceeded",
            StatusCode::BAD_REQUEST,
            log::Level::Warn,
        ));
    }

    // If adding as Group Leader, check if there's already a leader
    if body.role_id == AvailableStudentRole::GroupLeader as i32 {
        let is_leader = groups_repository::is_group_leader(&data.db, student.student_id, *group_id)
            .await
            .map_err(|e| {
                error_with_log_id(
                    format!("unable to check group leadership: {}", e),
                    "Database error",
                    StatusCode::INTERNAL_SERVER_ERROR,
                    log::Level::Error,
                )
            })?;

        if is_leader {
            return Err(error_with_log_id(
                "group already has a leader",
                "Group already has a leader",
                StatusCode::BAD_REQUEST,
                log::Level::Warn,
            ));
        }
    }

    // Add the student as a group member
    let mut member_state = DbState::new_uncreated(GroupMember {
        group_member_id: 0,
        group_id: *group_id,
        student_id: student.student_id,
        student_role_id: body.role_id,
        joined_at: Utc::now(),
    });

    match member_state.save(&data.db).await {
        Ok(_) => {
            let role_name = if body.role_id == AvailableStudentRole::GroupLeader as i32 {
                "Group Leader"
            } else {
                "Member"
            };

            Ok(HttpResponse::Created().json(AdminMemberResponse {
                success: true,
                message: "Member added successfully".to_string(),
                member: Some(AdminMemberInfo {
                    student_id: student.student_id,
                    name: format!("{} {}", student.first_name, student.last_name),
                    email: student.email,
                    role: role_name.to_string(),
                }),
            }))
        }
        Err(e) => Err(error_with_log_id(
            format!("unable to add student to group: {}", e),
            "Database error",
            StatusCode::INTERNAL_SERVER_ERROR,
            log::Level::Error,
        )),
    }
}
