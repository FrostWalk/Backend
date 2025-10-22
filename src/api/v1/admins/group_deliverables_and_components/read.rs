use crate::app_data::AppData;
use crate::common::json_error::{error_with_log_id, JsonError, ToJsonError};
use crate::database::repositories::group_deliverable_components_repository;
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
#[actix_web_grants::protect(any("ROLE_ADMIN_ROOT", "ROLE_ADMIN_PROFESSOR"))]
pub(super) async fn get_components_for_deliverable_handler(
    path: Path<i32>, data: Data<AppData>,
) -> Result<HttpResponse, JsonError> {
    let deliverable_id = path.into_inner();

    // Verify the group deliverable exists using repository function
    let deliverable_state = group_deliverables_repository::get_by_id(&data.db, deliverable_id)
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

    let deliverable = DbState::into_inner(deliverable_state);

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
            id: relationship_data.id,
            group_deliverable_id: relationship_data.group_deliverable_id,
            group_deliverable_component_id: relationship_data.group_deliverable_component_id,
            quantity: relationship_data.quantity,
            component_name: component.name.clone(),
            deliverable_name: deliverable.name.clone(),
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
#[actix_web_grants::protect(any("ROLE_ADMIN_ROOT", "ROLE_ADMIN_PROFESSOR"))]
pub(super) async fn get_deliverables_for_component_handler(
    path: Path<i32>, data: Data<AppData>,
) -> Result<HttpResponse, JsonError> {
    let component_id = path.into_inner();

    // Verify the group component exists using repository function
    let component_state =
        group_deliverable_components_repository::get_by_id(&data.db, component_id)
            .await
            .map_err(|e| {
                error_with_log_id(
                    format!("unable to check if group component exists: {}", e),
                    "Failed to retrieve deliverables",
                    StatusCode::INTERNAL_SERVER_ERROR,
                    log::Level::Error,
                )
            })?
            .ok_or_else(|| "Group component not found".to_json_error(StatusCode::NOT_FOUND))?;

    let component = DbState::into_inner(component_state);

    // Get deliverables with their details using repository function
    let deliverables_with_details =
        group_deliverables_components_repository::get_deliverables_with_details_for_component(
            &data.db,
            component_id,
        )
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

    for (relationship_state, deliverable_state) in deliverables_with_details {
        let relationship_data = DbState::into_inner(relationship_state);
        let deliverable = DbState::into_inner(deliverable_state);

        deliverables.push(GroupDeliverableComponentResponse {
            id: relationship_data.id,
            group_deliverable_id: relationship_data.group_deliverable_id,
            group_deliverable_component_id: relationship_data.group_deliverable_component_id,
            quantity: relationship_data.quantity,
            component_name: component.name.clone(),
            deliverable_name: deliverable.name.clone(),
        });
    }

    Ok(HttpResponse::Ok().json(GetDeliverablesForComponentResponse { deliverables }))
}
