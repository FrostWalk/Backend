use crate::app_data::AppData;
use crate::common::json_error::{error_with_log_id, JsonError};
use crate::database::repositories::projects_repository;
use crate::jwt::get_user::LoggedUser;
use crate::models::group_deliverable::GroupDeliverable;
use crate::models::group_deliverable_component::GroupDeliverableComponent;
use crate::models::project::Project;
use crate::models::student_deliverable::StudentDeliverable;
use crate::models::student_deliverable_component::StudentDeliverableComponent;
use actix_web::http::StatusCode;
use actix_web::web::Data;
use actix_web::{HttpMessage, HttpRequest, HttpResponse};
use serde::Serialize;
use utoipa::ToSchema;
use welds::state::DbState;

#[derive(Debug, Serialize, ToSchema)]
pub(crate) struct ProjectWithDetails {
    pub project: Project,
    pub group_deliverables: Vec<GroupDeliverable>,
    pub group_components: Vec<GroupDeliverableComponent>,
    pub student_deliverables: Vec<StudentDeliverable>,
    pub student_components: Vec<StudentDeliverableComponent>,
}

#[derive(Debug, Serialize, ToSchema)]
pub(crate) struct GetStudentProjects {
    projects: Vec<ProjectWithDetails>,
}

#[utoipa::path(
    get,
    path = "/v1/students/projects",
    responses(
        (status = 200, description = "Successfully retrieved student's projects with deliverables and components", body = GetStudentProjects),
        (status = 500, description = "Internal server error during serialization or database query", body = JsonError)
    ),
    security(("StudentAuth" = [])),
    tag = "Projects management",
)]
/// Get all the projects of student with deliverables and components
///
/// This endpoint allows authenticated students to retrieve all the projects in which they have a role,
/// along with all deliverables and components for each project
#[actix_web_grants::protect("ROLE_STUDENT")]
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

    // Fetch projects with all related entities using repository function
    let projects_with_details_data =
        projects_repository::get_projects_with_details_for_student(&data.db, user.student_id)
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

    let mut projects_with_details = Vec::new();

    for (
        project_state,
        group_deliverables_state,
        group_components_state,
        student_deliverables_state,
        student_components_state,
    ) in projects_with_details_data
    {
        let project = DbState::into_inner(project_state);
        let group_deliverables = group_deliverables_state
            .into_iter()
            .map(DbState::into_inner)
            .collect();
        let group_components = group_components_state
            .into_iter()
            .map(DbState::into_inner)
            .collect();
        let student_deliverables = student_deliverables_state
            .into_iter()
            .map(DbState::into_inner)
            .collect();
        let student_components = student_components_state
            .into_iter()
            .map(DbState::into_inner)
            .collect();

        projects_with_details.push(ProjectWithDetails {
            project,
            group_deliverables,
            group_components,
            student_deliverables,
            student_components,
        });
    }

    Ok(HttpResponse::Ok().json(GetStudentProjects {
        projects: projects_with_details,
    }))
}
