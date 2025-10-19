use crate::app_data::AppData;
use crate::common::json_error::{error_with_log_id, JsonError};
use crate::database::repositories::{
    admins_repository, coordinator_projects_repository, projects_repository,
};
use crate::jwt::get_user::LoggedUser;
use crate::models::admin_role::AvailableAdminRole;
use actix_web::http::StatusCode;
use actix_web::web::{Data, Json, Path};
use actix_web::{HttpMessage, HttpRequest, HttpResponse};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use welds::state::DbState;

#[derive(Debug, Deserialize, ToSchema)]
pub(crate) struct AssignCoordinatorRequest {
    pub admin_id: i32,
}

#[derive(Debug, Serialize, ToSchema)]
pub(crate) struct AssignCoordinatorResponse {
    pub message: String,
    pub coordinator_project_id: i32,
    pub coordinator: CoordinatorInfo,
    pub project: ProjectInfo,
}

#[derive(Debug, Serialize, ToSchema)]
pub(crate) struct CoordinatorInfo {
    pub admin_id: i32,
    pub name: String,
    pub email: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub(crate) struct ProjectInfo {
    pub project_id: i32,
    pub name: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub(crate) struct CoordinatorsListResponse {
    pub project_id: i32,
    pub project_name: String,
    pub coordinator: Option<CoordinatorDetail>,
}

#[derive(Debug, Serialize, ToSchema)]
pub(crate) struct CoordinatorDetail {
    pub admin_id: i32,
    pub first_name: String,
    pub last_name: String,
    pub email: String,
    pub assigned_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, ToSchema)]
pub(crate) struct RemoveCoordinatorResponse {
    pub message: String,
}

#[utoipa::path(
    post,
    path = "/v1/admins/projects/{project_id}/coordinators",
    request_body = AssignCoordinatorRequest,
    responses(
        (status = 201, description = "Coordinator assigned successfully", body = AssignCoordinatorResponse),
        (status = 400, description = "Invalid request or business rule violation", body = JsonError),
        (status = 401, description = "Authentication required", body = JsonError),
        (status = 404, description = "Project or admin not found", body = JsonError),
        (status = 500, description = "Internal server error", body = JsonError)
    ),
    security(("AdminAuth" = [])),
    tag = "Project Coordinators",
)]
/// Assign a coordinator to a project
///
/// This endpoint allows Professors and Root admins to assign Coordinators to specific projects.
/// Only admins with the Coordinator role can be assigned.
/// **Constraint**: At most one coordinator can be assigned per project.
pub(super) async fn assign_coordinator(
    req: HttpRequest, 
    path: Path<i32>,
    body: Json<AssignCoordinatorRequest>, 
    data: Data<AppData>,
) -> Result<HttpResponse, JsonError> {
    let _admin = match req.extensions().get_admin() {
        Ok(admin) => admin,
        Err(_) => {
            return Err(error_with_log_id(
                "entered a protected route without an admin loaded in the request",
                "Authentication error",
                StatusCode::INTERNAL_SERVER_ERROR,
                log::Level::Error,
            ));
        }
    };

    let project_id = path.into_inner();

    // Verify the project exists
    let project_state = projects_repository::get_by_id(&data.db, project_id)
        .await
        .map_err(|e| {
            error_with_log_id(
                format!("unable to fetch project {}: {}", project_id, e),
                "Database error",
                StatusCode::INTERNAL_SERVER_ERROR,
                log::Level::Error,
            )
        })?;

    let project = match project_state {
        Some(state) => DbState::into_inner(state),
        None => {
            return Err(error_with_log_id(
                format!("project {} not found", project_id),
                "Project not found",
                StatusCode::NOT_FOUND,
                log::Level::Warn,
            ));
        }
    };

    // Verify the admin exists
    let admin_state = admins_repository::get_by_id(&data.db, body.admin_id)
        .await
        .map_err(|e| {
            error_with_log_id(
                format!("unable to fetch admin {}: {}", body.admin_id, e),
                "Database error",
                StatusCode::INTERNAL_SERVER_ERROR,
                log::Level::Error,
            )
        })?;

    let admin = match admin_state {
        Some(state) => DbState::into_inner(state),
        None => {
            return Err(error_with_log_id(
                format!("admin {} not found", body.admin_id),
                "Admin not found",
                StatusCode::NOT_FOUND,
                log::Level::Warn,
            ));
        }
    };

    // Verify the admin is a Coordinator
    if admin.admin_role_id != AvailableAdminRole::Coordinator as i32 {
        return Err(error_with_log_id(
            format!(
                "admin {} is not a coordinator (role_id: {})",
                body.admin_id, admin.admin_role_id
            ),
            "Only Coordinators can be assigned to projects",
            StatusCode::BAD_REQUEST,
            log::Level::Warn,
        ));
    }

    // Check if the project already has a coordinator assigned
    let existing_coordinators =
        coordinator_projects_repository::get_by_project_id(&data.db, project_id)
            .await
            .map_err(|e| {
                error_with_log_id(
                    format!("unable to check existing coordinators: {}", e),
                    "Database error",
                    StatusCode::INTERNAL_SERVER_ERROR,
                    log::Level::Error,
                )
            })?;

    if !existing_coordinators.is_empty() {
        let existing_coordinator = &existing_coordinators[0];
        return Err(error_with_log_id(
            format!(
                "project {} already has a coordinator assigned (admin_id: {})",
                project_id, existing_coordinator.admin_id
            ),
            "Project can only have one coordinator. Remove the existing coordinator first.",
            StatusCode::BAD_REQUEST,
            log::Level::Warn,
        ));
    }

    // Create the assignment
    let assignment = coordinator_projects_repository::create(&data.db, body.admin_id, project_id)
        .await
        .map_err(|e| {
            error_with_log_id(
                format!("unable to create coordinator assignment: {}", e),
                "Database error",
                StatusCode::INTERNAL_SERVER_ERROR,
                log::Level::Error,
            )
        })?;

    let assignment = DbState::into_inner(assignment);

    Ok(HttpResponse::Created().json(AssignCoordinatorResponse {
        message: "Coordinator assigned to project successfully".to_string(),
        coordinator_project_id: assignment.coordinator_project_id,
        coordinator: CoordinatorInfo {
            admin_id: admin.admin_id,
            name: format!("{} {}", admin.first_name, admin.last_name),
            email: admin.email,
        },
        project: ProjectInfo {
            project_id: project.project_id,
            name: project.name,
        },
    }))
}

#[utoipa::path(
    get,
    path = "/v1/admins/projects/{project_id}/coordinators",
    responses(
        (status = 200, description = "List of coordinators for the project", body = CoordinatorsListResponse),
        (status = 401, description = "Authentication required", body = JsonError),
        (status = 404, description = "Project not found", body = JsonError),
        (status = 500, description = "Internal server error", body = JsonError)
    ),
    security(("AdminAuth" = [])),
    tag = "Project Coordinators",
)]
/// Get the coordinator assigned to a project
///
/// This endpoint allows admins to view the coordinator assigned to a specific project.
/// Returns null if no coordinator is assigned.
pub(super) async fn list_coordinators(
    req: HttpRequest, 
    path: Path<i32>, 
    data: Data<AppData>,
) -> Result<HttpResponse, JsonError> {
    let _admin = match req.extensions().get_admin() {
        Ok(admin) => admin,
        Err(_) => {
            return Err(error_with_log_id(
                "entered a protected route without an admin loaded in the request",
                "Authentication error",
                StatusCode::INTERNAL_SERVER_ERROR,
                log::Level::Error,
            ));
        }
    };

    let project_id = path.into_inner();

    // Verify the project exists
    let project_state = projects_repository::get_by_id(&data.db, project_id)
        .await
        .map_err(|e| {
            error_with_log_id(
                format!("unable to fetch project {}: {}", project_id, e),
                "Database error",
                StatusCode::INTERNAL_SERVER_ERROR,
                log::Level::Error,
            )
        })?;

    let project = match project_state {
        Some(state) => DbState::into_inner(state),
        None => {
            return Err(error_with_log_id(
                format!("project {} not found", project_id),
                "Project not found",
                StatusCode::NOT_FOUND,
                log::Level::Warn,
            ));
        }
    };

    // Get the coordinator assignment for this project (at most one)
    let assignments = coordinator_projects_repository::get_by_project_id(&data.db, project_id)
        .await
        .map_err(|e| {
            error_with_log_id(
                format!("unable to fetch coordinator assignment: {}", e),
                "Database error",
                StatusCode::INTERNAL_SERVER_ERROR,
                log::Level::Error,
            )
        })?;

    let coordinator = if let Some(assignment_state) = assignments.first() {
        let assignment = assignment_state;

        // Get admin details
        let admin_state = admins_repository::get_by_id(&data.db, assignment.admin_id)
            .await
            .map_err(|e| {
                error_with_log_id(
                    format!("unable to fetch admin {}: {}", assignment.admin_id, e),
                    "Database error",
                    StatusCode::INTERNAL_SERVER_ERROR,
                    log::Level::Error,
                )
            })?;

        if let Some(admin_state) = admin_state {
            let admin = DbState::into_inner(admin_state);
            Some(CoordinatorDetail {
                admin_id: admin.admin_id,
                first_name: admin.first_name,
                last_name: admin.last_name,
                email: admin.email,
                assigned_at: assignment.assigned_at,
            })
        } else {
            None
        }
    } else {
        None
    };

    Ok(HttpResponse::Ok().json(CoordinatorsListResponse {
        project_id,
        project_name: project.name,
        coordinator,
    }))
}

#[utoipa::path(
    delete,
    path = "/v1/admins/projects/{project_id}/coordinators/{admin_id}",
    responses(
        (status = 200, description = "Coordinator removed successfully", body = RemoveCoordinatorResponse),
        (status = 401, description = "Authentication required", body = JsonError),
        (status = 404, description = "Project or assignment not found", body = JsonError),
        (status = 500, description = "Internal server error", body = JsonError)
    ),
    security(("AdminAuth" = [])),
    tag = "Project Coordinators",
)]
/// Remove a coordinator from a project
///
/// This endpoint allows Professors and Root admins to remove coordinator assignments from projects.
pub(super) async fn remove_coordinator(
    req: HttpRequest, 
    path: Path<(i32, i32)>, 
    data: Data<AppData>,
) -> Result<HttpResponse, JsonError> {
    let _admin = match req.extensions().get_admin() {
        Ok(admin) => admin,
        Err(_) => {
            return Err(error_with_log_id(
                "entered a protected route without an admin loaded in the request",
                "Authentication error",
                StatusCode::INTERNAL_SERVER_ERROR,
                log::Level::Error,
            ));
        }
    };

    let (project_id, admin_id) = path.into_inner();

    // Verify the project exists
    let project_state = projects_repository::get_by_id(&data.db, project_id)
        .await
        .map_err(|e| {
            error_with_log_id(
                format!("unable to fetch project {}: {}", project_id, e),
                "Database error",
                StatusCode::INTERNAL_SERVER_ERROR,
                log::Level::Error,
            )
        })?;

    if project_state.is_none() {
        return Err(error_with_log_id(
            format!("project {} not found", project_id),
            "Project not found",
            StatusCode::NOT_FOUND,
            log::Level::Warn,
        ));
    }

    // Check if the assignment exists
    let is_assigned = coordinator_projects_repository::is_assigned(&data.db, admin_id, project_id)
        .await
        .map_err(|e| {
            error_with_log_id(
                format!("unable to check assignment: {}", e),
                "Database error",
                StatusCode::INTERNAL_SERVER_ERROR,
                log::Level::Error,
            )
        })?;

    if !is_assigned {
        return Err(error_with_log_id(
            format!(
                "coordinator {} is not assigned to project {}",
                admin_id, project_id
            ),
            "Coordinator not assigned to this project",
            StatusCode::NOT_FOUND,
            log::Level::Warn,
        ));
    }

    // Delete the assignment
    coordinator_projects_repository::delete(&data.db, admin_id, project_id)
        .await
        .map_err(|e| {
            error_with_log_id(
                format!("unable to delete coordinator assignment: {}", e),
                "Database error",
                StatusCode::INTERNAL_SERVER_ERROR,
                log::Level::Error,
            )
        })?;

    Ok(HttpResponse::Ok().json(RemoveCoordinatorResponse {
        message: "Coordinator removed from project successfully".to_string(),
    }))
}
