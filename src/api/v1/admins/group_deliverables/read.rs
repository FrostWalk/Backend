use crate::app_data::AppData;
use crate::common::json_error::{error_with_log_id, JsonError, ToJsonError};
use crate::database::repositories::group_deliverables_components_repository;
use crate::database::repositories::group_deliverables_repository;
use actix_web::http::StatusCode;
use actix_web::web::Data;
use actix_web::web::Path;
use actix_web::HttpResponse;
use serde::Serialize;
use utoipa::ToSchema;
use welds::state::DbState;

#[derive(Debug, Serialize, ToSchema)]
pub(crate) struct GroupDeliverableResponse {
    #[schema(example = "123")]
    pub group_deliverable_id: i32,
    #[schema(example = "1")]
    pub project_id: i32,
    #[schema(example = "Motor")]
    pub name: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub(crate) struct GetAllGroupDeliverablesResponse {
    pub deliverables: Vec<GroupDeliverableResponse>,
}

#[derive(Debug, Serialize, ToSchema)]
pub(crate) struct GetGroupDeliverablesForProjectResponse {
    pub deliverables: Vec<GroupDeliverableResponse>,
}

#[derive(Debug, Serialize, ToSchema)]
pub(crate) struct GroupDeliverableComponentResponse {
    #[schema(example = "1")]
    pub group_deliverable_id: i32,
    #[schema(example = "2")]
    pub group_deliverable_component_id: i32,
    #[schema(example = "5")]
    pub quantity: i32,
    #[schema(example = "Resistor")]
    pub component_name: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub(crate) struct GetComponentsForGroupDeliverableResponse {
    pub components: Vec<GroupDeliverableComponentResponse>,
}

#[utoipa::path(
    get,
    path = "/v1/admins/group-deliverables",
    responses(
        (status = 200, description = "Found all group deliverables", body = GetAllGroupDeliverablesResponse),
        (status = 500, description = "Internal server error occurred", body = JsonError)
    ),
    security(("AdminAuth" = [])),
    tag = "Group deliverables management",
)]
/// Get all group deliverables.
///
/// Returns all group deliverables across all projects.
#[actix_web_grants::protect(any("ROLE_ADMIN_ROOT", "ROLE_ADMIN_PROFESSOR"))]
pub(super) async fn get_all_group_deliverables_handler(
    data: Data<AppData>,
) -> Result<HttpResponse, JsonError> {
    let deliverables = group_deliverables_repository::get_all(&data.db)
        .await
        .map_err(|e| {
            error_with_log_id(
                format!("unable to retrieve all group deliverables: {}", e),
                "Failed to retrieve deliverables",
                StatusCode::INTERNAL_SERVER_ERROR,
                log::Level::Error,
            )
        })?;

    let response_deliverables: Vec<GroupDeliverableResponse> = deliverables
        .into_iter()
        .map(DbState::into_inner)
        .map(|deliverable| GroupDeliverableResponse {
            group_deliverable_id: deliverable.group_deliverable_id,
            project_id: deliverable.project_id,
            name: deliverable.name,
        })
        .collect();

    Ok(HttpResponse::Ok().json(GetAllGroupDeliverablesResponse {
        deliverables: response_deliverables,
    }))
}

#[utoipa::path(
    get,
    path = "/v1/admins/group-deliverables/project/{project_id}",
    responses(
        (status = 200, description = "Found group deliverables for project", body = GetGroupDeliverablesForProjectResponse),
        (status = 404, description = "Project not found", body = JsonError),
        (status = 500, description = "Internal server error occurred", body = JsonError)
    ),
    security(("AdminAuth" = [])),
    tag = "Group deliverables management",
)]
/// Get all group deliverables for a specific project.
///
/// Returns all group deliverables associated with the specified project.
#[actix_web_grants::protect(any("ROLE_ADMIN_ROOT", "ROLE_ADMIN_PROFESSOR"))]
pub(super) async fn get_group_deliverables_for_project_handler(
    path: Path<i32>, data: Data<AppData>,
) -> Result<HttpResponse, JsonError> {
    let project_id = path.into_inner();

    // Get all deliverables for this project
    let deliverables = group_deliverables_repository::get_by_project_id(&data.db, project_id)
        .await
        .map_err(|e| {
            error_with_log_id(
                format!(
                    "unable to retrieve deliverables for project {}: {}",
                    project_id, e
                ),
                "Failed to retrieve deliverables",
                StatusCode::INTERNAL_SERVER_ERROR,
                log::Level::Error,
            )
        })?;

    let mut response_deliverables = Vec::new();

    for deliverable in deliverables {
        let deliverable_data = DbState::into_inner(deliverable);
        response_deliverables.push(GroupDeliverableResponse {
            group_deliverable_id: deliverable_data.group_deliverable_id,
            project_id: deliverable_data.project_id,
            name: deliverable_data.name,
        });
    }

    Ok(
        HttpResponse::Ok().json(GetGroupDeliverablesForProjectResponse {
            deliverables: response_deliverables,
        }),
    )
}

#[utoipa::path(
    get,
    path = "/v1/admins/group-deliverables/{id}",
    responses(
        (status = 200, description = "Found group deliverable", body = GroupDeliverableResponse),
        (status = 404, description = "Group deliverable not found", body = JsonError),
        (status = 500, description = "Internal server error occurred", body = JsonError)
    ),
    security(("AdminAuth" = [])),
    tag = "Group deliverables management",
)]
/// Get a specific group deliverable by ID.
///
/// Returns the details of the specified group deliverable.
#[actix_web_grants::protect(any("ROLE_ADMIN_ROOT", "ROLE_ADMIN_PROFESSOR"))]
pub(super) async fn get_group_deliverable_handler(
    path: Path<i32>, data: Data<AppData>,
) -> Result<HttpResponse, JsonError> {
    let deliverable_id = path.into_inner();

    // Get the deliverable by ID
    let deliverable = group_deliverables_repository::get_by_id(&data.db, deliverable_id)
        .await
        .map_err(|e| {
            error_with_log_id(
                format!("unable to retrieve deliverable {}: {}", deliverable_id, e),
                "Failed to retrieve deliverable",
                StatusCode::INTERNAL_SERVER_ERROR,
                log::Level::Error,
            )
        })?
        .ok_or_else(|| "Group deliverable not found".to_json_error(StatusCode::NOT_FOUND))
        .map(DbState::into_inner)?;

    Ok(HttpResponse::Ok().json(GroupDeliverableResponse {
        group_deliverable_id: deliverable.group_deliverable_id,
        project_id: deliverable.project_id,
        name: deliverable.name,
    }))
}

#[utoipa::path(
    get,
    path = "/v1/admins/group-deliverables/{id}/components",
    responses(
        (status = 200, description = "Found components for group deliverable", body = GetComponentsForGroupDeliverableResponse),
        (status = 404, description = "Group deliverable not found", body = JsonError),
        (status = 500, description = "Internal server error occurred", body = JsonError)
    ),
    security(("AdminAuth" = [])),
    tag = "Group deliverables management",
)]
/// Get all components for a specific group deliverable.
///
/// Returns all components associated with the specified group deliverable along with their quantities.
#[actix_web_grants::protect(any(
    "ROLE_ADMIN_ROOT",
    "ROLE_ADMIN_PROFESSOR",
    "ROLE_ADMIN_COORDINATOR"
))]
pub(super) async fn get_components_for_group_deliverable_handler(
    path: Path<i32>, data: Data<AppData>,
) -> Result<HttpResponse, JsonError> {
    let deliverable_id = path.into_inner();

    // Verify the group deliverable exists
    let _deliverable = group_deliverables_repository::get_by_id(&data.db, deliverable_id)
        .await
        .map_err(|e| {
            error_with_log_id(
                format!("unable to check if group deliverable exists: {}", e),
                "Failed to retrieve components",
                StatusCode::INTERNAL_SERVER_ERROR,
                log::Level::Error,
            )
        })?
        .ok_or_else(|| "Group deliverable not found".to_json_error(StatusCode::NOT_FOUND))?;

    // Get components with their details using repository function
    let components_with_details =
        group_deliverables_components_repository::get_components_with_details_for_deliverable(
            &data.db,
            deliverable_id,
        )
        .await
        .map_err(|e| {
            error_with_log_id(
                format!(
                    "unable to retrieve components for deliverable {}: {}",
                    deliverable_id, e
                ),
                "Failed to retrieve components",
                StatusCode::INTERNAL_SERVER_ERROR,
                log::Level::Error,
            )
        })?;

    let mut components = Vec::new();

    for (relationship_state, component_state) in components_with_details {
        let relationship_data = DbState::into_inner(relationship_state);
        let component = DbState::into_inner(component_state);

        components.push(GroupDeliverableComponentResponse {
            group_deliverable_id: relationship_data.group_deliverable_id,
            group_deliverable_component_id: relationship_data.group_deliverable_component_id,
            quantity: relationship_data.quantity,
            component_name: component.name,
        });
    }

    Ok(HttpResponse::Ok().json(GetComponentsForGroupDeliverableResponse { components }))
}
