use crate::app_data::AppData;
use crate::common::json_error::{error_with_log_id, JsonError, ToJsonError};
use crate::models::group_deliverable::GroupDeliverable;
use crate::models::group_deliverables_component::GroupDeliverablesComponent;
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
    let deliverables = GroupDeliverable::all().run(&data.db).await.map_err(|e| {
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
    let deliverables = GroupDeliverable::where_col(|gd| gd.project_id.equal(project_id))
        .run(&data.db)
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
    let mut deliverables =
        GroupDeliverable::where_col(|gd| gd.group_deliverable_id.equal(deliverable_id))
            .run(&data.db)
            .await
            .map_err(|e| {
                error_with_log_id(
                    format!("unable to retrieve deliverable {}: {}", deliverable_id, e),
                    "Failed to retrieve deliverable",
                    StatusCode::INTERNAL_SERVER_ERROR,
                    log::Level::Error,
                )
            })?;

    let deliverable = match deliverables.pop() {
        Some(p) => DbState::into_inner(p),
        None => return Err("Group deliverable not found".to_json_error(StatusCode::NOT_FOUND)),
    };

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
    let deliverable_exists =
        GroupDeliverable::where_col(|gd| gd.group_deliverable_id.equal(deliverable_id))
            .run(&data.db)
            .await
            .map_err(|e| {
                error_with_log_id(
                    format!("unable to check if group deliverable exists: {}", e),
                    "Failed to retrieve components",
                    StatusCode::INTERNAL_SERVER_ERROR,
                    log::Level::Error,
                )
            })?;

    if deliverable_exists.is_empty() {
        return Err("Group deliverable not found".to_json_error(StatusCode::NOT_FOUND));
    }

    // Efficiently fetch relationships with related entities using map_query
    let relationships =
        GroupDeliverablesComponent::where_col(|gdc| gdc.group_deliverable_id.equal(deliverable_id))
            .run(&data.db)
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

    let components_data =
        GroupDeliverablesComponent::where_col(|gdc| gdc.group_deliverable_id.equal(deliverable_id))
            .map_query(|gdc| gdc.group_deliverable_component)
            .run(&data.db)
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

    for (relationship_state, component_state) in relationships.into_iter().zip(components_data) {
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
