use crate::app_data::AppData;
use crate::common::json_error::{error_with_log_id, JsonError};
use crate::jwt::get_user::LoggedUser;
use crate::models::group::Group;
use crate::models::group_member::GroupMember;
use crate::models::student_role::AvailableStudentRole;
use actix_web::http::StatusCode;
use actix_web::web::{Data, Json, Path};
use actix_web::{HttpMessage, HttpRequest, HttpResponse};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use welds::state::DbState;

#[derive(Debug, Deserialize, ToSchema)]
pub(crate) struct UpdateGroupRequest {
    pub name: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub(crate) struct UpdateGroupResponse {
    pub group: Group,
}

#[utoipa::path(
    put,
    path = "/v1/students/groups/{group_id}",
    request_body = UpdateGroupRequest,
    responses(
        (status = 200, description = "Group updated successfully", body = UpdateGroupResponse),
        (status = 400, description = "Invalid request data or business rule violation", body = JsonError),
        (status = 401, description = "Authentication required", body = JsonError),
        (status = 403, description = "Insufficient permissions", body = JsonError),
        (status = 404, description = "Group not found", body = JsonError),
        (status = 500, description = "Internal server error", body = JsonError)
    ),
    security(("UserAuth" = [])),
    tag = "Groups management",
)]
/// Update a group
///
/// This endpoint allows authenticated students with GroupLeader role to update a group they lead.
pub(super) async fn update_group(
    req: HttpRequest, data: Data<AppData>, path: Path<i32>, body: Json<UpdateGroupRequest>,
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

    // Verify the group exists and the user is a GroupLeader of this group
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

    // Check if the user is a GroupLeader of this group
    let mut is_group_leader = false;
    for gm in group_member_states {
        let group_member = DbState::into_inner(gm);
        if group_member.student_id == user.student_id
            && group_member.student_role_id == AvailableStudentRole::GroupLeader as i32
        {
            is_group_leader = true;
            break;
        }
    }

    if !is_group_leader {
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

    // Get the current group
    let mut group_states = Group::where_col(|g| g.group_id.equal(group_id))
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

    let mut group_state = match group_states.pop() {
        Some(state) => state,
        None => {
            return Err(error_with_log_id(
                format!("group {} not found", group_id),
                "Group not found",
                StatusCode::NOT_FOUND,
                log::Level::Warn,
            ));
        }
    };

    // Update the group name
    group_state.name = body.name.clone();

    // Save the updated group
    group_state.save(&data.db).await.map_err(|e| {
        error_with_log_id(
            format!("unable to update group {}: {}", group_id, e),
            "Database error",
            StatusCode::INTERNAL_SERVER_ERROR,
            log::Level::Error,
        )
    })?;

    let updated_group = DbState::into_inner(group_state);

    Ok(HttpResponse::Ok().json(UpdateGroupResponse {
        group: updated_group,
    }))
}
