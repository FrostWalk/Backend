use crate::app_data::AppData;
use crate::common::json_error::{database_error, JsonError, ToJsonError};
use crate::jwt::get_user::LoggedUser;
use actix_web::http::StatusCode;
use actix_web::web::Data;
use actix_web::{HttpMessage, HttpRequest, HttpResponse};
use entity::projects::Model;
use log::error;
use serde::Serialize;
use utoipa::ToSchema;

#[derive(Debug, Serialize, ToSchema)]
pub(crate) struct GetStudentProjects {
    projects: Vec<Model>,
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
        Err(e) => {
            error!("entered a protected route without a user loaded in the request");
            return Err(e.to_json_error(StatusCode::INTERNAL_SERVER_ERROR));
        }
    };

    let projects = match data
        .repositories
        .projects
        .find_projects_for_student(user.student_id)
        .await
    {
        Ok(p) => p,
        Err(e) => {
            error!("unable to retrieve projects of user from database: {}", e);
            return Err(database_error());
        }
    };

    Ok(HttpResponse::Ok().json(GetStudentProjects { projects }))
}
