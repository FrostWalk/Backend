use crate::app_data::AppData;
use crate::common::json_error::{error_with_log_id, JsonError, ToJsonError};
use crate::models::group_deliverable_component::GroupDeliverableComponent;
use crate::models::group_deliverables_component::GroupDeliverablesComponent;
use actix_web::http::StatusCode;
use actix_web::web::Data;
use actix_web::web::Path;
use actix_web::HttpResponse;
use serde::Serialize;
use utoipa::ToSchema;
use welds::state::DbState;

#[derive(Debug, Serialize, ToSchema)]
pub(crate) struct GroupComponentResponse {
    #[schema(example = "123")]
    pub group_deliverable_component_id: i32,
    #[schema(example = "1")]
    pub project_id: i32,
    #[schema(example = "Resistor")]
    pub name: String,
    #[schema(example = "true")]
    pub sellable: bool,
}

#[derive(Debug, Serialize, ToSchema)]
pub(crate) struct GetAllGroupComponentsResponse {
    pub components: Vec<GroupComponentResponse>,
}

#[derive(Debug, Serialize, ToSchema)]
pub(crate) struct GetGroupComponentsForProjectResponse {
    pub components: Vec<GroupComponentResponse>,
}

#[derive(Debug, Serialize, ToSchema)]
pub(crate) struct GroupComponentDeliverableResponse {
    #[schema(example = "1")]
    pub group_deliverable_id: i32,
    #[schema(example = "2")]
    pub group_deliverable_component_id: i32,
    #[schema(example = "5")]
    pub quantity: i32,
    #[schema(example = "Motor")]
    pub deliverable_name: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub(crate) struct GetDeliverablesForGroupComponentResponse {
    pub deliverables: Vec<GroupComponentDeliverableResponse>,
}

#[utoipa::path(
    get,
    path = "/v1/admins/group-deliverable-components",
    responses(
        (status = 200, description = "Found all group components", body = GetAllGroupComponentsResponse),
        (status = 500, description = "Internal server error occurred", body = JsonError)
    ),
    security(("AdminAuth" = [])),
    tag = "Group deliverable components management",
)]
/// Get all group components.
///
/// Returns all group components across all projects.
#[actix_web_grants::protect(any("ROLE_ADMIN_ROOT", "ROLE_ADMIN_PROFESSOR"))]
pub(super) async fn get_all_group_components_handler(
    data: Data<AppData>,
) -> Result<HttpResponse, JsonError> {
    let components = GroupDeliverableComponent::all()
        .run(&data.db)
        .await
        .map_err(|e| {
            error_with_log_id(
                format!("unable to retrieve all group components: {}", e),
                "Failed to retrieve components",
                StatusCode::INTERNAL_SERVER_ERROR,
                log::Level::Error,
            )
        })?;

    let response_components: Vec<GroupComponentResponse> = components
        .into_iter()
        .map(DbState::into_inner)
        .map(|component| GroupComponentResponse {
            group_deliverable_component_id: component.group_deliverable_component_id,
            project_id: component.project_id,
            name: component.name,
            sellable: component.sellable,
        })
        .collect();

    Ok(HttpResponse::Ok().json(GetAllGroupComponentsResponse {
        components: response_components,
    }))
}

#[utoipa::path(
    get,
    path = "/v1/admins/group-deliverable-components/project/{project_id}",
    responses(
        (status = 200, description = "Found group components for project", body = GetGroupComponentsForProjectResponse),
        (status = 404, description = "Project not found", body = JsonError),
        (status = 500, description = "Internal server error occurred", body = JsonError)
    ),
    security(("AdminAuth" = [])),
    tag = "Group deliverable components management",
)]
/// Get all group components for a specific project.
///
/// Returns all group components associated with the specified project.
#[actix_web_grants::protect(any("ROLE_ADMIN_ROOT", "ROLE_ADMIN_PROFESSOR"))]
pub(super) async fn get_group_components_for_project_handler(
    path: Path<i32>, data: Data<AppData>,
) -> Result<HttpResponse, JsonError> {
    let project_id = path.into_inner();

    // Get all components for this project
    let components = GroupDeliverableComponent::where_col(|gc| gc.project_id.equal(project_id))
        .run(&data.db)
        .await
        .map_err(|e| {
            error_with_log_id(
                format!(
                    "unable to retrieve components for project {}: {}",
                    project_id, e
                ),
                "Failed to retrieve components",
                StatusCode::INTERNAL_SERVER_ERROR,
                log::Level::Error,
            )
        })?;

    let mut response_components = Vec::new();

    for component in components {
        let component_data = DbState::into_inner(component);
        response_components.push(GroupComponentResponse {
            group_deliverable_component_id: component_data.group_deliverable_component_id,
            project_id: component_data.project_id,
            name: component_data.name,
            sellable: component_data.sellable,
        });
    }

    Ok(
        HttpResponse::Ok().json(GetGroupComponentsForProjectResponse {
            components: response_components,
        }),
    )
}

#[utoipa::path(
    get,
    path = "/v1/admins/group-deliverable-components/{id}",
    responses(
        (status = 200, description = "Found group component", body = GroupComponentResponse),
        (status = 404, description = "Group component not found", body = JsonError),
        (status = 500, description = "Internal server error occurred", body = JsonError)
    ),
    security(("AdminAuth" = [])),
    tag = "Group deliverable components management",
)]
/// Get a specific group component by ID.
///
/// Returns the details of the specified group component.
#[actix_web_grants::protect(any("ROLE_ADMIN_ROOT", "ROLE_ADMIN_PROFESSOR"))]
pub(super) async fn get_group_component_handler(
    path: Path<i32>, data: Data<AppData>,
) -> Result<HttpResponse, JsonError> {
    let component_id = path.into_inner();

    // Get the component by ID
    let mut components = GroupDeliverableComponent::where_col(|gc| {
        gc.group_deliverable_component_id.equal(component_id)
    })
    .run(&data.db)
    .await
    .map_err(|e| {
        error_with_log_id(
            format!("unable to retrieve component {}: {}", component_id, e),
            "Failed to retrieve component",
            StatusCode::INTERNAL_SERVER_ERROR,
            log::Level::Error,
        )
    })?;

    let component = match components.pop() {
        Some(c) => DbState::into_inner(c),
        None => return Err("Group component not found".to_json_error(StatusCode::NOT_FOUND)),
    };

    Ok(HttpResponse::Ok().json(GroupComponentResponse {
        group_deliverable_component_id: component.group_deliverable_component_id,
        project_id: component.project_id,
        name: component.name,
        sellable: component.sellable,
    }))
}

#[utoipa::path(
    get,
    path = "/v1/admins/group-deliverable-components/{id}/deliverables",
    responses(
        (status = 200, description = "Found deliverables for group component", body = GetDeliverablesForGroupComponentResponse),
        (status = 404, description = "Group component not found", body = JsonError),
        (status = 500, description = "Internal server error occurred", body = JsonError)
    ),
    security(("AdminAuth" = [])),
    tag = "Group deliverable components management",
)]
/// Get all deliverables that use a specific group component.
///
/// Returns all group deliverables that use the specified component along with their quantities.
#[actix_web_grants::protect(any("ROLE_ADMIN_ROOT", "ROLE_ADMIN_PROFESSOR"))]
pub(super) async fn get_deliverables_for_group_component_handler(
    path: Path<i32>, data: Data<AppData>,
) -> Result<HttpResponse, JsonError> {
    let component_id = path.into_inner();

    // Verify the group component exists
    let component_exists = GroupDeliverableComponent::where_col(|gc| {
        gc.group_deliverable_component_id.equal(component_id)
    })
    .run(&data.db)
    .await
    .map_err(|e| {
        error_with_log_id(
            format!("unable to check if group component exists: {}", e),
            "Failed to retrieve deliverables",
            StatusCode::INTERNAL_SERVER_ERROR,
            log::Level::Error,
        )
    })?;

    if component_exists.is_empty() {
        return Err("Group component not found".to_json_error(StatusCode::NOT_FOUND));
    }

    // Efficiently fetch relationships with related entities using map_query
    let relationships = GroupDeliverablesComponent::where_col(|gdc| {
        gdc.group_deliverable_component_id.equal(component_id)
    })
    .run(&data.db)
    .await
    .map_err(|e| {
        error_with_log_id(
            format!(
                "unable to retrieve deliverables for component {}: {}",
                component_id, e
            ),
            "Failed to retrieve deliverables",
            StatusCode::INTERNAL_SERVER_ERROR,
            log::Level::Error,
        )
    })?;

    let deliverables_data = GroupDeliverablesComponent::where_col(|gdc| {
        gdc.group_deliverable_component_id.equal(component_id)
    })
    .map_query(|gdc| gdc.group_deliverable)
    .run(&data.db)
    .await
    .map_err(|e| {
        error_with_log_id(
            format!(
                "unable to retrieve deliverables for component {}: {}",
                component_id, e
            ),
            "Failed to retrieve deliverables",
            StatusCode::INTERNAL_SERVER_ERROR,
            log::Level::Error,
        )
    })?;

    let mut deliverables = Vec::new();

    for (relationship_state, deliverable_state) in relationships.into_iter().zip(deliverables_data)
    {
        let relationship_data = DbState::into_inner(relationship_state);
        let deliverable = DbState::into_inner(deliverable_state);

        deliverables.push(GroupComponentDeliverableResponse {
            group_deliverable_id: relationship_data.group_deliverable_id,
            group_deliverable_component_id: relationship_data.group_deliverable_component_id,
            quantity: relationship_data.quantity,
            deliverable_name: deliverable.name,
        });
    }

    Ok(HttpResponse::Ok().json(GetDeliverablesForGroupComponentResponse { deliverables }))
}
