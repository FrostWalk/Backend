use crate::app_data::AppData;
use crate::common::json_error::{error_with_log_id, JsonError, ToJsonError};
use crate::database::repositories::projects_repository;
use crate::models::group_deliverable::GroupDeliverable;
use crate::models::group_deliverable_component::GroupDeliverableComponent;
use crate::models::project::Project;
use crate::models::student_deliverable::StudentDeliverable;
use crate::models::student_deliverable_component::StudentDeliverableComponent;
use actix_web::http::StatusCode;
use actix_web::web::{Data, Path};
use actix_web::HttpResponse;
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
pub(in crate::api::v1) async fn get_all_projects_handler(
    data: Data<AppData>,
) -> Result<HttpResponse, JsonError> {
    let states = projects_repository::get_all(&data.db).await.map_err(|e| {
        error_with_log_id(
            format!("unable to retrieve projects from database: {}", e),
            "Failed to retrieve projects",
            StatusCode::INTERNAL_SERVER_ERROR,
            log::Level::Error,
        )
    })?;

    let projects: Vec<Project> = states
        .into_iter()
        .map(welds::state::DbState::into_inner)
        .collect();

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
        (status = 404, description = "project not found", body = JsonError),
        (status = 500, description = "Internal server error", body = JsonError)
    ),
    security(("AdminAuth" = [])),
    tag = "Projects management",
)]
/// Get project details by id with deliverables and components
pub(in crate::api::v1) async fn get_one_project_handler(
    path: Path<i32>, data: Data<AppData>,
) -> Result<HttpResponse, JsonError> {
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
