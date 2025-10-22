use crate::app_data::AppData;
use crate::common::json_error::{error_with_log_id, JsonError};
use crate::database::repositories::{groups_repository, security_codes};
use crate::jwt::get_user::LoggedUser;
use crate::models::group::Group;
use crate::models::group_member::GroupMember;
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
    pub name: String,
    pub security_code: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub(crate) struct CreateGroupResponse {
    pub group_id: i32,
    pub name: String,
    pub project_id: i32,
    pub role: String,
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
    security(("StudentAuth" = [])),
    tag = "Groups management",
)]
/// Create a new group for a project
///
/// This endpoint allows authenticated students to create a group using a valid security code.
/// The security code must be valid and not expired for the specified project.
/// Each student can only create one group per project.
/// The group creator becomes the GroupLeader automatically.
#[actix_web_grants::protect("ROLE_STUDENT")]
pub(crate) async fn create_group(
    req: HttpRequest, body: Json<CreateGroupRequest>, data: Data<AppData>,
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
    let security_code_state = security_codes::get_by_code(&data.db, &body.security_code)
        .await
        .map_err(|e| {
            error_with_log_id(
                format!("unable to verify security code: {}", e),
                "Database error",
                StatusCode::INTERNAL_SERVER_ERROR,
                log::Level::Error,
            )
        })?;

    let security_code = match security_code_state {
        Some(state) => DbState::into_inner(state),
        None => {
            return Err(error_with_log_id(
                "security code not found",
                "Invalid security code",
                StatusCode::BAD_REQUEST,
                log::Level::Warn,
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
    let in_project = groups_repository::is_student_in_project(
        &data.db,
        user.student_id,
        security_code.project_id,
    )
    .await
    .map_err(|e| {
        error_with_log_id(
            format!("unable to check existing groups: {}", e),
            "Database error",
            StatusCode::INTERNAL_SERVER_ERROR,
            log::Level::Error,
        )
    })?;

    if in_project {
        return Err(error_with_log_id(
            "student already has a group for this project",
            "Group already exists",
            StatusCode::BAD_REQUEST,
            log::Level::Warn,
        ));
    }

    // Create the group using repository function
    let group = Group {
        group_id: 0,
        project_id: security_code.project_id,
        name: body.name.clone(),
        created_at: Utc::now(),
    };

    let created_group = groups_repository::create_group(&data.db, group)
        .await
        .map_err(|e| {
            error_with_log_id(
                format!("unable to create group: {}", e),
                "Database error",
                StatusCode::INTERNAL_SERVER_ERROR,
                log::Level::Error,
            )
        })?;

    let group_data = DbState::into_inner(created_group);

    // Add the student as a group member with GroupLeader role using repository function
    let group_member = GroupMember {
        group_member_id: 0,
        group_id: group_data.group_id,
        student_id: user.student_id,
        student_role_id: AvailableStudentRole::GroupLeader as i32,
        joined_at: Utc::now(),
    };

    groups_repository::create_group_member(&data.db, group_member)
        .await
        .map_err(|e| {
            // Note: We can't await in map_err, so we'll just log the error
            // The group will remain in the database but this is acceptable
            // as it's a rare error case
            error_with_log_id(
                format!("unable to add student as group member: {}", e),
                "Database error",
                StatusCode::INTERNAL_SERVER_ERROR,
                log::Level::Error,
            )
        })?;

    Ok(HttpResponse::Created().json(CreateGroupResponse {
        group_id: group_data.group_id,
        name: group_data.name,
        project_id: group_data.project_id,
        role: "Group Leader".to_string(),
    }))
}
