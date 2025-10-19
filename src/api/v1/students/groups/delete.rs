use crate::app_data::AppData;
use crate::common::json_error::{error_with_log_id, JsonError};
use crate::database::repositories::groups_repository;
use crate::jwt::get_user::LoggedUser;
use actix_web::http::StatusCode;
use actix_web::web::{Data, Path};
use actix_web::{HttpMessage, HttpRequest, HttpResponse};
use utoipa::ToSchema;

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
    security(("StudentAuth" = [])),
    tag = "Groups management",
)]
/// Delete a group
///
/// This endpoint allows authenticated students to delete a group they lead.
/// This will also remove all group members.
pub(crate) async fn delete_group(
    req: HttpRequest, 
    path: Path<i32>, 
    data: Data<AppData>,
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

    // Delete the group and all its members
    groups_repository::delete_group_with_members(&data.db, group_id)
        .await
        .map_err(|e| {
            error_with_log_id(
                format!("unable to delete group {}: {}", group_id, e),
                "Database error",
                StatusCode::INTERNAL_SERVER_ERROR,
                log::Level::Error,
            )
        })?;

    Ok(HttpResponse::Ok().json(DeleteGroupResponse {
        message: format!("Group {} deleted successfully", group_id),
    }))
}
