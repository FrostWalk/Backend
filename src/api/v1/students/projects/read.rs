use crate::app_data::AppData;
use crate::common::json_error::{error_with_log_id, JsonError};
use crate::jwt::get_user::LoggedUser;
use crate::models::group_deliverable::GroupDeliverable;
use crate::models::group_deliverable_component::GroupDeliverableComponent;
use crate::models::group_member::GroupMember;
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

    // Fetch projects with all related entities efficiently using map_query
    let projects: Vec<Project> = GroupMember::where_col(|gm| gm.student_id.equal(user.student_id))
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
        })?
        .into_iter()
        .map(DbState::into_inner)
        .collect();

    let mut projects_with_details = Vec::new();

    for project in projects {
        let project_id = project.project_id;

        // Fetch all related entities efficiently using map_query
        let group_deliverables: Vec<GroupDeliverable> =
            Project::where_col(|p| p.project_id.equal(project_id))
                .map_query(|p| p.group_deliverables)
                .run(&data.db)
                .await
                .map_err(|e| {
                    error_with_log_id(
                        format!("unable to retrieve group deliverables: {}", e),
                        "Failed to retrieve project details",
                        StatusCode::INTERNAL_SERVER_ERROR,
                        log::Level::Error,
                    )
                })?
                .into_iter()
                .map(DbState::into_inner)
                .collect();

        let group_components: Vec<GroupDeliverableComponent> =
            Project::where_col(|p| p.project_id.equal(project_id))
                .map_query(|p| p.group_deliverable_components)
                .run(&data.db)
                .await
                .map_err(|e| {
                    error_with_log_id(
                        format!("unable to retrieve group components: {}", e),
                        "Failed to retrieve project details",
                        StatusCode::INTERNAL_SERVER_ERROR,
                        log::Level::Error,
                    )
                })?
                .into_iter()
                .map(DbState::into_inner)
                .collect();

        let student_deliverables: Vec<StudentDeliverable> =
            Project::where_col(|p| p.project_id.equal(project_id))
                .map_query(|p| p.student_deliverables)
                .run(&data.db)
                .await
                .map_err(|e| {
                    error_with_log_id(
                        format!("unable to retrieve student deliverables: {}", e),
                        "Failed to retrieve project details",
                        StatusCode::INTERNAL_SERVER_ERROR,
                        log::Level::Error,
                    )
                })?
                .into_iter()
                .map(DbState::into_inner)
                .collect();

        let student_components: Vec<StudentDeliverableComponent> =
            Project::where_col(|p| p.project_id.equal(project_id))
                .map_query(|p| p.student_deliverable_components)
                .run(&data.db)
                .await
                .map_err(|e| {
                    error_with_log_id(
                        format!("unable to retrieve student components: {}", e),
                        "Failed to retrieve project details",
                        StatusCode::INTERNAL_SERVER_ERROR,
                        log::Level::Error,
                    )
                })?
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
