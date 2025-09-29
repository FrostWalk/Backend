use crate::app_data::AppData;
use crate::common::json_error::{error_with_log_id, JsonError};
use crate::jwt::get_user::LoggedUser;
use crate::models::group::Group;
use crate::models::group_member::GroupMember;
use crate::models::security_code::SecurityCode;
use crate::models::student_role::AvailableStudentRole;
use actix_web::http::StatusCode;
use actix_web::web::{Data, Json};
use actix_web::{HttpMessage, HttpRequest, HttpResponse};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use utoipa::ToSchema;
use welds::state::DbState;

#[derive(Debug, Deserialize, ToSchema)]
pub(crate) struct CreateGroupRequest {
    pub name: String,
    pub security_code: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub(crate) struct CreateGroupResponse {
    pub group: Group,
}

#[utoipa::path(
    post,
    path = "/v1/students/groups",
    request_body = CreateGroupRequest,
    responses(
        (status = 201, description = "Group created successfully", body = CreateGroupResponse),
        (status = 400, description = "Invalid request data", body = JsonError),
        (status = 401, description = "Authentication required", body = JsonError),
        (status = 409, description = "User already has a group for this project", body = JsonError),
        (status = 500, description = "Internal server error", body = JsonError)
    ),
    security(("UserAuth" = [])),
    tag = "Groups management",
)]
/// Create a new group for a project
///
/// This endpoint allows authenticated students to create a group using a valid security code.
/// The security code must be valid and not expired for the specified project.
/// Each student can only create one group per project.
/// The group creator becomes the GroupLeader automatically.
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

    // Verify the security code is valid and extract project_id
    let security_code = match SecurityCode::where_col(|sc| sc.code.equal(&body.security_code))
        .run(&data.db)
        .await
    {
        Ok(mut rows) => match rows.pop() {
            Some(state) => DbState::into_inner(state),
            None => {
                return Err(error_with_log_id(
                    "security code not found",
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

    // Validate security code expiration
    if security_code.expiration <= Utc::now() {
        return Err(error_with_log_id(
            "security code has expired",
            "Invalid security code",
            StatusCode::BAD_REQUEST,
            log::Level::Warn,
        ));
    }

    // Check if the student already has a group for this project
    // Get all groups for this project
    let groups = match Group::where_col(|g| g.project_id.equal(security_code.project_id))
        .run(&data.db)
        .await
    {
        Ok(rows) => rows
            .into_iter()
            .map(DbState::into_inner)
            .map(|g| g.group_id)
            .collect::<HashSet<i32>>(),
        Err(e) => {
            return Err(error_with_log_id(
                format!("unable to fetch groups for project: {}", e),
                "Database error",
                StatusCode::INTERNAL_SERVER_ERROR,
                log::Level::Error,
            ));
        }
    };

    if groups.is_empty() {
        // No groups exist for this project, so student can't be in any
    } else {
        // Check if student is a member of any group in this project
        let memberships = match GroupMember::where_col(|gm| gm.student_id.equal(user.student_id))
            .run(&data.db)
            .await
        {
            Ok(rows) => rows
                .into_iter()
                .map(DbState::into_inner)
                .map(|gm| gm.group_id)
                .collect::<HashSet<i32>>(),
            Err(e) => {
                return Err(error_with_log_id(
                    format!("unable to check existing groups: {}", e),
                    "User already has a group",
                    StatusCode::CONFLICT,
                    log::Level::Error,
                ));
            }
        };

        // Check if student is a member of any group in this project
        if !groups.is_disjoint(&memberships) {
            return Err(error_with_log_id(
                "student already has a group for this project",
                "Group already exists",
                StatusCode::BAD_REQUEST,
                log::Level::Warn,
            ));
        }
    }

    // Create the group
    let mut group_state = DbState::new_uncreated(Group {
        group_id: 0,
        project_id: security_code.project_id,
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
        group_member_id: 0,
        group_id: created_group.group_id,
        student_id: user.student_id,
        student_role_id: AvailableStudentRole::GroupLeader as i32,
    });

    match group_member_state.save(&data.db).await {
        Ok(_) => {}
        Err(e) => {
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

    Ok(HttpResponse::Created().json(CreateGroupResponse {
        group: created_group,
    }))
}
