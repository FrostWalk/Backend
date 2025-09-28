use crate::app_data::AppData;
use crate::common::json_error::{error_with_log_id, JsonError};
use crate::jwt::get_user::LoggedUser;
use crate::models::group_member::GroupMember;
use crate::models::project::Project;
use actix_web::http::StatusCode;
use actix_web::web::Data;
use actix_web::{HttpMessage, HttpRequest, HttpResponse};
use serde::Serialize;
use utoipa::ToSchema;
use welds::state::DbState;

#[derive(Debug, Serialize, ToSchema)]
pub(crate) struct GetStudentProjects {
    projects: Vec<Project>,
}
#[utoipa::path(
    get,
    path = "/v1/students/projects",
    responses(
        (status = 200, description = "Successfully retrieved student's projects", body = GetStudentProjects),
        (status = 500, description = "Internal server error during serialization or database query", body = JsonError)
    ),
    security(("UserAuth" = [])),
    tag = "Projects management",
)]
/// Get all the projects of student
///
/// This endpoint allows authenticated students to retrieve all the projects in which has a role
pub(super) async fn get_student_projects(
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

    let project_states = GroupMember::where_col(|gm| gm.student_id.equal(user.student_id))
        .map_query(|gm| gm.group)
        .map_query(|g| g.project)
        .run(&data.db)
        .await
        .map_err(|e| {
            error_with_log_id(
                format!(
                    "unable to fetch student projects from database {}: {}",
                    user.student_id, e
                ),
                "Failed to retrieve projects",
                StatusCode::INTERNAL_SERVER_ERROR,
                log::Level::Error,
            )
        })?;

    let projects: Vec<Project> = project_states
        .into_iter()
        .map(DbState::into_inner)
        .collect();

    Ok(HttpResponse::Ok().json(GetStudentProjects { projects }))
}
