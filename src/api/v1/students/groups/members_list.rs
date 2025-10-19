use crate::app_data::AppData;
use crate::common::json_error::{error_with_log_id, JsonError};
use crate::database::repositories::{groups_repository, students_repository};
use crate::jwt::get_user::LoggedUser;
use actix_web::http::StatusCode;
use actix_web::web::{Data, Path};
use actix_web::{HttpMessage, HttpRequest, HttpResponse};
use serde::Serialize;
use utoipa::ToSchema;
use welds::state::DbState;

#[derive(Debug, Serialize, ToSchema)]
pub(crate) struct GroupMembersResponse {
    pub group_id: i32,
    pub group_name: String,
    pub members: Vec<GroupMemberInfo>,
}

#[derive(Debug, Serialize, ToSchema)]
pub(crate) struct GroupMemberInfo {
    pub student_id: i32,
    pub first_name: String,
    pub last_name: String,
    pub email: String,
    pub role_id: i32,
    pub role_name: String,
}

#[utoipa::path(
    get,
    path = "/v1/students/groups/{group_id}/members",
    responses(
        (status = 200, description = "Group members list", body = GroupMembersResponse),
        (status = 401, description = "Authentication required", body = JsonError),
        (status = 404, description = "Group not found", body = JsonError),
        (status = 500, description = "Internal server error", body = JsonError)
    ),
    security(("StudentAuth" = [])),
    tag = "Group management",
)]
/// List all members of a group with their roles
///
/// This endpoint allows students to view all members of a group with their roles.
/// Any authenticated student can view group members.
pub(super) async fn list_group_members(
    req: HttpRequest, 
    path: Path<i32>, 
    data: Data<AppData>,
) -> Result<HttpResponse, JsonError> {
    let _user = match req.extensions().get_student() {
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

    // Verify the group exists
    let group_state = groups_repository::get_by_id(&data.db, group_id)
        .await
        .map_err(|e| {
            error_with_log_id(
                format!("unable to fetch group: {}", e),
                "Database error",
                StatusCode::INTERNAL_SERVER_ERROR,
                log::Level::Error,
            )
        })?;

    let group = match group_state {
        Some(state) => DbState::into_inner(state),
        None => {
            return Err(error_with_log_id(
                format!("group with id {} not found", group_id),
                "Group not found",
                StatusCode::NOT_FOUND,
                log::Level::Warn,
            ));
        }
    };

    // Get all group members
    let group_members = groups_repository::get_group_members(&data.db, group_id)
        .await
        .map_err(|e| {
            error_with_log_id(
                format!("unable to fetch group members: {}", e),
                "Database error",
                StatusCode::INTERNAL_SERVER_ERROR,
                log::Level::Error,
            )
        })?;

    // Get student details for each member
    let mut members = Vec::new();
    for group_member in group_members {
        let student_state = students_repository::get_by_id(&data.db, group_member.student_id)
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
            let role_name = match group_member.student_role_id {
                1 => "Group Leader",
                2 => "Member",
                _ => "Unknown",
            };

            members.push(GroupMemberInfo {
                student_id: student.student_id,
                first_name: student.first_name,
                last_name: student.last_name,
                email: student.email,
                role_id: group_member.student_role_id,
                role_name: role_name.to_string(),
            });
        }
    }

    Ok(HttpResponse::Ok().json(GroupMembersResponse {
        group_id: group.group_id,
        group_name: group.name,
        members,
    }))
}
