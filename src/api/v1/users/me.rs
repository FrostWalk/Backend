use crate::app_state::AppState;
use crate::common::json_error::{JsonError, ToJsonError};
use crate::database::repositories::projects_repository::ProjectsAndRoles;
use actix_web::error::{ErrorInternalServerError, ErrorNotFound};
use actix_web::web::Data;
use actix_web::{Error, HttpMessage, HttpRequest, HttpResponse};
use derive_new::new;
use entity::users;
use sea_orm::prelude::DateTime;
use serde::Serialize;
use utoipa::ToSchema;

/// Schema for user profile with associated projects and roles
#[derive(Debug, Serialize, ToSchema)]
pub(crate) struct UserProjectsSchema {
    /// Unique user ID
    #[schema(example = 1)]
    id: i32,
    /// User's first name
    #[schema(example = "John")]
    name: String,
    /// User's last name
    #[schema(example = "Doe")]
    surname: String,
    /// User's email address
    #[schema(example = "john.doe@example.com")]
    email: String,
    /// Optional student ID
    #[schema(example = 12345)]
    student_id: Option<i32>,
    /// Optional Telegram username
    #[schema(example = "johndoe")]
    telegram_nick: Option<String>,
    /// List of projects the user is associated with and their roles
    projects: Vec<ProjectRole>,
}

/// Schema for a project and the user's role in it
#[derive(Debug, Serialize, ToSchema, new)]
struct ProjectRole {
    /// Unique project ID
    #[schema(example = 1)]
    id: i32,
    /// Project name
    #[schema(example = "Project Alpha")]
    name: String,
    /// Project year
    #[schema(example = 2023)]
    year: i16,
    /// User's role in the project
    #[schema(example = "Developer")]
    role_name: String,
    /// Whether the user has retired from the project
    #[schema(example = false)]
    has_retired: bool,
    /// Date of retirement (if applicable)
    #[schema(example = "2023-01-15T00:00:00")]
    retirement_date: Option<DateTime>,
}

#[utoipa::path(
    get,
    path = "/v1/users/me",
    responses(
        (status = 200, description = "Successfully retrieved user profile", body = UserProjectsSchema),
        (status = 404, description = "User not found in request context", body = JsonError),
        (status = 500, description = "Internal server error during serialization or database query", body = JsonError)
    ),
    tag = "Users",
)]
/// Returns authenticated user's profile information
///
/// Extracts user data from request extensions (set by auth middleware),
/// filters sensitive fields, and returns user data with associated projects and roles.
pub(super) async fn me_handler(
    req: HttpRequest, app_state: Data<AppState>,
) -> Result<HttpResponse, Error> {
    // extract user from request, loaded by auth middleware
    let user = match req.extensions().get::<users::Model>() {
        None => return Err(ErrorNotFound("user does not exists".to_json_error())),
        Some(user) => user.clone(),
    };

    // find all the projects in which user took part and his role in it
    let projects_roles = to_projects_roles(
        app_state
            .repositories
            .project_repository
            .get_user_projects(user.id)
            .await
            .map_err(ErrorInternalServerError)?,
    );

    Ok(HttpResponse::Ok().json(UserProjectsSchema::from(user, projects_roles)))
}
#[inline(always)]
fn to_projects_roles(p: ProjectsAndRoles) -> Vec<ProjectRole> {
    let mut result: Vec<ProjectRole> = Vec::with_capacity(p.len());
    for i in 0..p.len() {
        result[i] = ProjectRole::new(
            p[i].0.id,
            p[i].0.name.clone(),
            p[i].0.year,
            p[i].1.name.clone(),
            p[i].2 .0,
            p[i].2 .1,
        );
    }
    result
}
impl UserProjectsSchema {
    #[inline(always)]
    fn from(u: users::Model, p: Vec<ProjectRole>) -> Self {
        Self {
            id: u.id,
            name: u.name,
            surname: u.surname,
            student_id: u.student_id,
            email: u.email,
            telegram_nick: u.telegram_nick,
            projects: p,
        }
    }
}
