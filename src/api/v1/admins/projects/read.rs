use crate::app_data::AppData;
use crate::common::json_error::{error_with_log_id, JsonError, ToJsonError};
use crate::database::repositories::coordinator_projects_repository;
use crate::database::repositories::projects_repository;
use crate::jwt::get_user::LoggedUser;
use crate::models::admin_role::AvailableAdminRole;
use crate::models::group_deliverable::GroupDeliverable;
use crate::models::group_deliverable_component::GroupDeliverableComponent;
use crate::models::project::Project;
use crate::models::student_deliverable::StudentDeliverable;
use crate::models::student_deliverable_component::StudentDeliverableComponent;
use actix_web::http::StatusCode;
use actix_web::web::{Data, Path};
use actix_web::{HttpMessage, HttpRequest, HttpResponse};
use log::error;
use serde::Serialize;
use utoipa::ToSchema;
use welds::state::DbState;

#[derive(Debug, Serialize, ToSchema)]
pub(crate) struct GetAllProjectsResponse {
    projects: Vec<Project>,
}
#[utoipa::path(
    get,
    path = "/v1/admins/projects",
    responses(
        (status = 200, description = "Found projects", body = GetAllProjectsResponse),
        (status = 500, description = "Internal server error occurred", body = JsonError)
    ),
    security(("AdminAuth" = [])),
    tag = "Projects management",
)]
/// Get all projects details
///
/// Returns all projects for Professors/Root, or only assigned projects for Coordinators
#[actix_web_grants::protect(any(
    "ROLE_ADMIN_ROOT",
    "ROLE_ADMIN_PROFESSOR",
    "ROLE_ADMIN_COORDINATOR"
))]
pub(in crate::api::v1) async fn get_all_projects_handler(
    req: HttpRequest, data: Data<AppData>,
) -> Result<HttpResponse, JsonError> {
    let user = match req.extensions().get_admin() {
        Ok(user) => user,
        Err(e) => {
            error!("entered a protected route without a user loaded in the request");
            return Err(e.to_json_error(StatusCode::INTERNAL_SERVER_ERROR));
        }
    };

    // Check if user is a coordinator
    let is_coordinator = user.admin_role_id == AvailableAdminRole::Coordinator as i32;

    let projects: Vec<Project> = if is_coordinator {
        // Coordinators see only their assigned projects
        let project_ids =
            coordinator_projects_repository::get_projects_by_coordinator(&data.db, user.admin_id)
                .await
                .map_err(|e| {
                    error_with_log_id(
                        format!("unable to retrieve coordinator projects: {}", e),
                        "Failed to retrieve projects",
                        StatusCode::INTERNAL_SERVER_ERROR,
                        log::Level::Error,
                    )
                })?;

        if project_ids.is_empty() {
            Vec::new()
        } else {
            // Fetch projects by IDs
            let states = projects_repository::get_all(&data.db).await.map_err(|e| {
                error_with_log_id(
                    format!("unable to retrieve projects from database: {}", e),
                    "Failed to retrieve projects",
                    StatusCode::INTERNAL_SERVER_ERROR,
                    log::Level::Error,
                )
            })?;

            states
                .into_iter()
                .map(DbState::into_inner)
                .filter(|p| project_ids.contains(&p.project_id))
                .collect()
        }
    } else {
        // Professors and Root see all projects
        let states = projects_repository::get_all(&data.db).await.map_err(|e| {
            error_with_log_id(
                format!("unable to retrieve projects from database: {}", e),
                "Failed to retrieve projects",
                StatusCode::INTERNAL_SERVER_ERROR,
                log::Level::Error,
            )
        })?;

        states
            .into_iter()
            .map(welds::state::DbState::into_inner)
            .collect()
    };

    Ok(HttpResponse::Ok().json(GetAllProjectsResponse { projects }))
}

#[derive(Debug, Serialize, ToSchema)]
pub(crate) struct ProjectDetailsResponse {
    pub project: Project,
    pub group_deliverables: Vec<GroupDeliverable>,
    pub group_components: Vec<GroupDeliverableComponent>,
    pub student_deliverables: Vec<StudentDeliverable>,
    pub student_components: Vec<StudentDeliverableComponent>,
}

#[utoipa::path(
    get,
    path = "/v1/admins/projects/{id}",
    responses(
        (status = 200, description = "Found project with deliverables and components", body = ProjectDetailsResponse),
        (status = 403, description = "Access denied", body = JsonError),
        (status = 404, description = "project not found", body = JsonError),
        (status = 500, description = "Internal server error", body = JsonError)
    ),
    security(("AdminAuth" = [])),
    tag = "Projects management",
)]
/// Get project details by id with deliverables and components
///
/// Coordinators can only view projects they are assigned to. Professors/Root can view any project.
#[actix_web_grants::protect(any(
    "ROLE_ADMIN_ROOT",
    "ROLE_ADMIN_PROFESSOR",
    "ROLE_ADMIN_COORDINATOR"
))]
pub(in crate::api::v1) async fn get_one_project_handler(
    req: HttpRequest, path: Path<i32>, data: Data<AppData>,
) -> Result<HttpResponse, JsonError> {
    let user = match req.extensions().get_admin() {
        Ok(user) => user,
        Err(e) => {
            error!("entered a protected route without a user loaded in the request");
            return Err(e.to_json_error(StatusCode::INTERNAL_SERVER_ERROR));
        }
    };

    let id = path.into_inner();

    let state = projects_repository::get_by_id(&data.db, id)
        .await
        .map_err(|e| {
            error_with_log_id(
                format!("database error: {}", e),
                "Failed to retrieve projects",
                StatusCode::INTERNAL_SERVER_ERROR,
                log::Level::Error,
            )
        })?;

    let proj = match state {
        Some(state) => DbState::into_inner(state),
        None => return Err("Project not found".to_json_error(StatusCode::NOT_FOUND)),
    };

    // Check if user is a coordinator and if they have access to this project
    let is_coordinator = user.admin_role_id == AvailableAdminRole::Coordinator as i32;
    if is_coordinator {
        let is_assigned = coordinator_projects_repository::is_assigned(&data.db, user.admin_id, id)
            .await
            .map_err(|e| {
                error_with_log_id(
                    format!("unable to check coordinator assignment: {}", e),
                    "Failed to retrieve project",
                    StatusCode::INTERNAL_SERVER_ERROR,
                    log::Level::Error,
                )
            })?;

        if !is_assigned {
            return Err("Access denied - you are not assigned to this project"
                .to_json_error(StatusCode::FORBIDDEN));
        }
    }

    // Fetch all related entities efficiently using map_query
    let group_deliverables: Vec<GroupDeliverable> = Project::where_col(|p| p.project_id.equal(id))
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
        Project::where_col(|p| p.project_id.equal(id))
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
        Project::where_col(|p| p.project_id.equal(id))
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
        Project::where_col(|p| p.project_id.equal(id))
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

    Ok(HttpResponse::Ok().json(ProjectDetailsResponse {
        project: proj,
        group_deliverables,
        group_components,
        student_deliverables,
        student_components,
    }))
}
