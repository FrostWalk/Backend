use crate::app_data::AppData;
use crate::common::json_error::{error_with_log_id, JsonError};
use crate::jwt::get_user::LoggedUser;
use crate::models::group::Group;
use crate::models::group_member::GroupMember;
use crate::models::student_role::AvailableStudentRole;
use actix_web::http::StatusCode;
use actix_web::web::{Data, Path};
use actix_web::{HttpMessage, HttpRequest, HttpResponse};
use utoipa::ToSchema;
use welds::state::DbState;

#[derive(Debug, serde::Serialize, ToSchema)]
pub(crate) struct DeleteGroupResponse {
    pub message: String,
}

#[utoipa::path(
    delete,
    path = "/v1/students/groups/{group_id}",
    responses(
        (status = 200, description = "Group deleted successfully", body = DeleteGroupResponse),
        (status = 401, description = "Authentication required", body = JsonError),
        (status = 403, description = "Insufficient permissions", body = JsonError),
        (status = 500, description = "Internal server error", body = JsonError)
    ),
    security(("UserAuth" = [])),
    tag = "Groups management",
)]
/// Delete a group
///
/// This endpoint allows authenticated students to delete a group they lead.
/// This will also remove all group members.
pub(crate) async fn delete_group(
    req: HttpRequest, data: Data<AppData>, path: Path<i32>,
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

    // Delete all group members first
    match GroupMember::where_col(|gm| gm.group_id.equal(group_id))
        .delete(&data.db)
        .await
    {
        Ok(_) => {}
        Err(e) => {
            return Err(error_with_log_id(
                format!(
                    "unable to delete group members for group {}: {}",
                    group_id, e
                ),
                "Database error",
                StatusCode::INTERNAL_SERVER_ERROR,
                log::Level::Error,
            ));
        }
    }

    // Delete the group
    match Group::where_col(|g| g.group_id.equal(group_id))
        .delete(&data.db)
        .await
    {
        Ok(_) => {}
        Err(e) => {
            return Err(error_with_log_id(
                format!("unable to delete group {}: {}", group_id, e),
                "Database error",
                StatusCode::INTERNAL_SERVER_ERROR,
                log::Level::Error,
            ));
        }
    }

    Ok(HttpResponse::Ok().json(DeleteGroupResponse {
        message: format!("Group {} deleted successfully", group_id),
    }))
}
