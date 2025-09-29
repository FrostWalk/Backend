use crate::app_data::AppData;
use crate::common::json_error::{error_with_log_id, JsonError};
use crate::jwt::get_user::LoggedUser;
use crate::models::group::Group;
use crate::models::group_member::GroupMember;
use crate::models::project::Project;
use crate::models::security_code::SecurityCode;
use crate::models::student::Student;
use crate::models::student_role::AvailableStudentRole;
use actix_web::http::StatusCode;
use actix_web::web::{Data, Json};
use actix_web::{HttpMessage, HttpRequest, HttpResponse};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use welds::state::DbState;

#[derive(Debug, Deserialize, ToSchema)]
pub(crate) struct CreateGroupRequest {
    pub project_id: i32,
    pub name: String,
    pub security_code: String,
    pub member_emails: Vec<String>,
}

#[derive(Debug, Serialize, ToSchema)]
pub(crate) struct CreateGroupResponse {
    pub group: Group,
    pub members_added: Vec<MemberInfo>,
    pub members_not_found: Vec<String>,
}

#[derive(Debug, Serialize, ToSchema)]
pub(crate) struct MemberInfo {
    pub student_id: i32,
    pub email: String,
    pub first_name: String,
    pub last_name: String,
}

#[utoipa::path(
    post,
    path = "/v1/students/groups",
    request_body = CreateGroupRequest,
    responses(
        (status = 201, description = "Group created successfully", body = CreateGroupResponse),
        (status = 400, description = "Invalid request data or business rule violation", body = JsonError),
        (status = 401, description = "Authentication required", body = JsonError),
        (status = 500, description = "Internal server error", body = JsonError)
    ),
    security(("UserAuth" = [])),
    tag = "Groups management",
)]
/// Create a new group for a project
///
/// This endpoint allows authenticated students with GroupLeader role to create a group.
/// The student must provide a valid security code with GroupLeader role for the specified project.
/// Each student can only create one group per project.
/// The endpoint also accepts member emails to add students with Member role to the group.
pub(crate) async fn create_group(
    req: HttpRequest, data: Data<AppData>, body: Json<CreateGroupRequest>,
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

    // Verify the project exists
    let _project = match Project::where_col(|p| p.project_id.equal(body.project_id))
        .run(&data.db)
        .await
    {
        Ok(mut rows) => match rows.pop() {
            Some(state) => DbState::into_inner(state),
            None => {
                return Err(error_with_log_id(
                    format!("project with id {} not found", body.project_id),
                    "Project not found",
                    StatusCode::BAD_REQUEST,
                    log::Level::Warn,
                ));
            }
        },
        Err(e) => {
            return Err(error_with_log_id(
                format!("unable to fetch project from database: {}", e),
                "Database error",
                StatusCode::INTERNAL_SERVER_ERROR,
                log::Level::Error,
            ));
        }
    };

    // Verify the security code is valid and has GroupLeader role
    let security_code = match SecurityCode::where_col(|sc| sc.code.equal(&body.security_code))
        .run(&data.db)
        .await
    {
        Ok(mut rows) => match rows.pop() {
            Some(state) => DbState::into_inner(state),
            None => {
                return Err(error_with_log_id(
                    "invalid or expired security code for GroupLeader role",
                    "Invalid security code",
                    StatusCode::BAD_REQUEST,
                    log::Level::Warn,
                ));
            }
        },
        Err(e) => {
            return Err(error_with_log_id(
                format!("unable to verify security code: {}", e),
                "Database error",
                StatusCode::INTERNAL_SERVER_ERROR,
                log::Level::Error,
            ));
        }
    };

    // Validate security code properties
    if security_code.project_id != body.project_id {
        return Err(error_with_log_id(
            "security code is not valid for this project",
            "Invalid security code",
            StatusCode::BAD_REQUEST,
            log::Level::Warn,
        ));
    }

    if security_code.student_role_id != AvailableStudentRole::GroupLeader as i32 {
        return Err(error_with_log_id(
            "security code is not valid for GroupLeader role",
            "Invalid security code",
            StatusCode::BAD_REQUEST,
            log::Level::Warn,
        ));
    }

    if security_code.expiration <= Utc::now() {
        return Err(error_with_log_id(
            "security code has expired",
            "Invalid security code",
            StatusCode::BAD_REQUEST,
            log::Level::Warn,
        ));
    }

    // Check if the student already has a group for this project
    let existing_groups = match GroupMember::where_col(|gm| gm.student_id.equal(user.student_id))
        .run(&data.db)
        .await
    {
        Ok(rows) => rows,
        Err(e) => {
            return Err(error_with_log_id(
                format!("unable to check existing groups: {}", e),
                "Database error",
                StatusCode::INTERNAL_SERVER_ERROR,
                log::Level::Error,
            ));
        }
    };

    // Check if any of the existing groups are for this project
    for group_member in existing_groups {
        let group_member_data = DbState::into_inner(group_member);
        let group_states = Group::where_col(|g| g.group_id.equal(group_member_data.group_id))
            .run(&data.db)
            .await
            .map_err(|e| {
                error_with_log_id(
                    format!(
                        "unable to fetch group {}: {}",
                        group_member_data.group_id, e
                    ),
                    "Database error",
                    StatusCode::INTERNAL_SERVER_ERROR,
                    log::Level::Error,
                )
            })?;

        if let Some(group_state) = group_states.into_iter().next() {
            let group = DbState::into_inner(group_state);
            if group.project_id == body.project_id {
                return Err(error_with_log_id(
                    "student already has a group for this project",
                    "Group already exists",
                    StatusCode::BAD_REQUEST,
                    log::Level::Warn,
                ));
            }
        }
    }

    // Create the group
    let mut group_state = DbState::new_uncreated(Group {
        group_id: 0, // Will be set by the database
        project_id: body.project_id,
        name: body.name.clone(),
    });

    let created_group = match group_state.save(&data.db).await {
        Ok(_) => DbState::into_inner(group_state),
        Err(e) => {
            return Err(error_with_log_id(
                format!("unable to create group: {}", e),
                "Database error",
                StatusCode::INTERNAL_SERVER_ERROR,
                log::Level::Error,
            ));
        }
    };

    // Add the student as a group member with GroupLeader role
    let mut group_member_state = DbState::new_uncreated(GroupMember {
        group_member_id: 0, // Will be set by the database
        group_id: created_group.group_id,
        student_id: user.student_id,
        student_role_id: AvailableStudentRole::GroupLeader as i32,
    });

    match group_member_state.save(&data.db).await {
        Ok(_) => {}
        Err(e) => {
            // If adding the member fails, we should clean up the group
            let _ = Group::where_col(|g| g.group_id.equal(created_group.group_id))
                .delete(&data.db)
                .await;

            return Err(error_with_log_id(
                format!("unable to add student as group member: {}", e),
                "Database error",
                StatusCode::INTERNAL_SERVER_ERROR,
                log::Level::Error,
            ));
        }
    }

    // Add members to the group
    let mut members_added = Vec::new();
    let mut members_not_found = Vec::new();

    for email in &body.member_emails {
        // Find the student by email
        let student_states = match Student::where_col(|s| s.email.equal(email))
            .run(&data.db)
            .await
        {
            Ok(rows) => rows,
            Err(e) => {
                log::warn!("unable to find student with email {}: {}", email, e);
                members_not_found.push(email.clone());
                continue;
            }
        };

        if let Some(student_state) = student_states.into_iter().next() {
            let student = DbState::into_inner(student_state);

            // Check if the student is already in a group for this project
            let existing_membership =
                match GroupMember::where_col(|gm| gm.student_id.equal(student.student_id))
                    .run(&data.db)
                    .await
                {
                    Ok(rows) => rows,
                    Err(e) => {
                        log::warn!(
                            "unable to check existing membership for student {}: {}",
                            student.student_id,
                            e
                        );
                        members_not_found.push(email.clone());
                        continue;
                    }
                };

            // Check if any of the existing memberships are for this project
            let mut already_in_project = false;
            for membership in existing_membership {
                let membership_data = DbState::into_inner(membership);
                let group_states = Group::where_col(|g| g.group_id.equal(membership_data.group_id))
                    .run(&data.db)
                    .await
                    .unwrap_or_default();

                if let Some(group_state) = group_states.into_iter().next() {
                    let group = DbState::into_inner(group_state);
                    if group.project_id == body.project_id {
                        already_in_project = true;
                        break;
                    }
                }
            }

            if already_in_project {
                log::warn!(
                    "student {} is already in a group for project {}",
                    student.student_id,
                    body.project_id
                );
                members_not_found.push(email.clone());
                continue;
            }

            // Add the student as a group member with Member role
            let mut member_state = DbState::new_uncreated(GroupMember {
                group_member_id: 0,
                group_id: created_group.group_id,
                student_id: student.student_id,
                student_role_id: AvailableStudentRole::Member as i32,
            });

            match member_state.save(&data.db).await {
                Ok(_) => {
                    members_added.push(MemberInfo {
                        student_id: student.student_id,
                        email: student.email.clone(),
                        first_name: student.first_name,
                        last_name: student.last_name,
                    });
                }
                Err(e) => {
                    log::warn!(
                        "unable to add student {} to group: {}",
                        student.student_id,
                        e
                    );
                    members_not_found.push(email.clone());
                }
            }
        } else {
            members_not_found.push(email.clone());
        }
    }

    Ok(HttpResponse::Created().json(CreateGroupResponse {
        group: created_group,
        members_added,
        members_not_found,
    }))
}
