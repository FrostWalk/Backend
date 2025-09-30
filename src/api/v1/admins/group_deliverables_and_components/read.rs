use crate::app_data::AppData;
use crate::common::json_error::{error_with_log_id, JsonError, ToJsonError};
use crate::models::group_deliverable::GroupDeliverable;
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
pub(crate) struct GroupDeliverableComponentResponse {
    #[schema(example = "123")]
    pub id: i32,
    #[schema(example = "1")]
    pub group_deliverable_id: i32,
    #[schema(example = "2")]
    pub group_deliverable_component_id: i32,
    #[schema(example = "5")]
    pub quantity: i32,
    #[schema(example = "Resistor")]
    pub component_name: String,
    #[schema(example = "10k")]
    pub deliverable_name: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub(crate) struct GetComponentsForDeliverableResponse {
    pub components: Vec<GroupDeliverableComponentResponse>,
}

#[derive(Debug, Serialize, ToSchema)]
pub(crate) struct GetDeliverablesForComponentResponse {
    pub deliverables: Vec<GroupDeliverableComponentResponse>,
}

#[utoipa::path(
    get,
    path = "/v1/admins/group-deliverables-components/components/{deliverable_id}",
    responses(
        (status = 200, description = "Found components for group deliverable", body = GetComponentsForDeliverableResponse),
        (status = 404, description = "Group deliverable not found", body = JsonError),
        (status = 500, description = "Internal server error occurred", body = JsonError)
    ),
    security(("AdminAuth" = [])),
    tag = "Group deliverables-components management",
)]
/// Get all components for a specific group deliverable.
///
/// Returns all components associated with the specified group deliverable along with their quantities.
pub(super) async fn get_components_for_deliverable_handler(
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

    // Get all relationships for this deliverable
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

    let mut components = Vec::new();

    for relationship in relationships {
        let relationship_data = DbState::into_inner(relationship);

        // Get component details
        let mut component_rows = GroupDeliverableComponent::where_col(|gc| {
            gc.group_deliverable_component_id
                .equal(relationship_data.group_deliverable_component_id)
        })
        .run(&data.db)
        .await
        .map_err(|e| {
            error_with_log_id(
                format!("unable to retrieve component details: {}", e),
                "Failed to retrieve components",
                StatusCode::INTERNAL_SERVER_ERROR,
                log::Level::Error,
            )
        })?;

        let component = match component_rows.pop() {
            Some(c) => DbState::into_inner(c),
            None => continue, // Skip if deliverable component not found
        };

        // Get deliverable details
        let mut deliverable_rows = GroupDeliverable::where_col(|gd| {
            gd.group_deliverable_id
                .equal(relationship_data.group_deliverable_id)
        })
        .run(&data.db)
        .await
        .map_err(|e| {
            error_with_log_id(
                format!("unable to retrieve deliverable details: {}", e),
                "Failed to retrieve components",
                StatusCode::INTERNAL_SERVER_ERROR,
                log::Level::Error,
            )
        })?;

        let deliverable = match deliverable_rows.pop() {
            Some(p) => DbState::into_inner(p),
            None => continue, // Skip if deliverable not found
        };

        components.push(GroupDeliverableComponentResponse {
            id: relationship_data.id,
            group_deliverable_id: relationship_data.group_deliverable_id,
            group_deliverable_component_id: relationship_data.group_deliverable_component_id,
            quantity: relationship_data.quantity,
            component_name: component.name,
            deliverable_name: deliverable.name,
        });
    }

    Ok(HttpResponse::Ok().json(GetComponentsForDeliverableResponse { components }))
}

#[utoipa::path(
    get,
    path = "/v1/admins/group-deliverables-components/deliverables/{component_id}",
    responses(
        (status = 200, description = "Found deliverables for group component", body = GetDeliverablesForComponentResponse),
        (status = 404, description = "Group deliverable component not found", body = JsonError),
        (status = 500, description = "Internal server error occurred", body = JsonError)
    ),
    security(("AdminAuth" = [])),
    tag = "Group deliverables-components management",
)]
/// Get all deliverables that use a specific group component.
///
/// Returns all group deliverables that use the specified component along with their quantities.
pub(super) async fn get_deliverables_for_component_handler(
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

    // Get all relationships for this component
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

    let mut deliverables = Vec::new();

    for relationship in relationships {
        let relationship_data = DbState::into_inner(relationship);

        // Get component details
        let mut component_rows = GroupDeliverableComponent::where_col(|gc| {
            gc.group_deliverable_component_id
                .equal(relationship_data.group_deliverable_component_id)
        })
        .run(&data.db)
        .await
        .map_err(|e| {
            error_with_log_id(
                format!("unable to retrieve component details: {}", e),
                "Failed to retrieve deliverables",
                StatusCode::INTERNAL_SERVER_ERROR,
                log::Level::Error,
            )
        })?;

        let component = match component_rows.pop() {
            Some(c) => DbState::into_inner(c),
            None => continue, // Skip if component not found
        };

        // Get deliverable details
        let mut deliverable_rows = GroupDeliverable::where_col(|gd| {
            gd.group_deliverable_id
                .equal(relationship_data.group_deliverable_id)
        })
        .run(&data.db)
        .await
        .map_err(|e| {
            error_with_log_id(
                format!("unable to retrieve deliverable details: {}", e),
                "Failed to retrieve deliverables",
                StatusCode::INTERNAL_SERVER_ERROR,
                log::Level::Error,
            )
        })?;

        let deliverable = match deliverable_rows.pop() {
            Some(p) => DbState::into_inner(p),
            None => continue, // Skip if deliverable not found
        };

        deliverables.push(GroupDeliverableComponentResponse {
            id: relationship_data.id,
            group_deliverable_id: relationship_data.group_deliverable_id,
            group_deliverable_component_id: relationship_data.group_deliverable_component_id,
            quantity: relationship_data.quantity,
            component_name: component.name,
            deliverable_name: deliverable.name,
        });
    }

    Ok(HttpResponse::Ok().json(GetDeliverablesForComponentResponse { deliverables }))
}
