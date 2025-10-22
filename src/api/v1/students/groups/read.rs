use crate::app_data::AppData;
use crate::common::json_error::{error_with_log_id, JsonError};
use crate::database::repositories::groups_repository;
use crate::jwt::get_user::LoggedUser;
use crate::models::group::Group;
use crate::models::project::Project;
use actix_web::http::StatusCode;
use actix_web::web::Data;
use actix_web::{HttpMessage, HttpRequest, HttpResponse};
use serde::Serialize;
use utoipa::ToSchema;
use welds::state::DbState;

#[derive(Debug, Serialize, ToSchema)]
pub(crate) struct GetGroupsResponse {
    pub groups: Vec<GroupWithProject>,
}

#[derive(Debug, Serialize, ToSchema)]
pub(crate) struct GroupWithProject {
    pub group: Group,
    pub project: Project,
}

#[utoipa::path(
    get,
    path = "/v1/students/groups",
    responses(
        (status = 200, description = "Successfully retrieved student's groups", body = GetGroupsResponse),
        (status = 401, description = "Authentication required", body = JsonError),
        (status = 500, description = "Internal server error", body = JsonError)
    ),
    security(("StudentAuth" = [])),
    tag = "Groups management",
)]
/// Get all groups where the student is a member
///
/// This endpoint allows authenticated students to retrieve all groups they are members of.
#[actix_web_grants::protect("ROLE_STUDENT")]
pub(crate) async fn get_groups(
    req: HttpRequest, data: Data<AppData>,
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

    // Get all groups with their projects efficiently using repository
    let groups_and_projects =
        groups_repository::get_groups_with_projects_for_student(&data.db, user.student_id)
            .await
            .map_err(|e| {
                error_with_log_id(
                    format!(
                        "unable to fetch student groups from database {}: {}",
                        user.student_id, e
                    ),
                    "Failed to retrieve groups",
                    StatusCode::INTERNAL_SERVER_ERROR,
                    log::Level::Error,
                )
            })?;

    let mut groups_with_projects = Vec::new();

    for (_group_member_state, group_state, project_state) in groups_and_projects {
        let group = DbState::into_inner(group_state);
        let project = DbState::into_inner(project_state);

        groups_with_projects.push(GroupWithProject { group, project });
    }

    Ok(HttpResponse::Ok().json(GetGroupsResponse {
        groups: groups_with_projects,
    }))
}
