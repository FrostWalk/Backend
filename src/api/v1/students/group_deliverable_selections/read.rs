use crate::app_data::AppData;
use crate::common::json_error::{error_with_log_id, JsonError};
use crate::database::repositories::{
    group_component_implementation_details_repository, group_deliverable_components_repository,
    group_deliverable_selections_repository, group_deliverables_repository,
};
use actix_web::http::StatusCode;
use actix_web::web::{Data, Path};
use actix_web::HttpResponse;
use serde::Serialize;
use utoipa::ToSchema;
use welds::state::DbState;

#[derive(Debug, Serialize, ToSchema)]
pub(crate) struct ComponentImplementationDetail {
    pub id: i32,
    pub group_deliverable_component_id: i32,
    pub component_name: String,
    pub markdown_description: String,
    pub repository_link: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Serialize, ToSchema)]
pub(crate) struct GroupDeliverableSelectionResponse {
    pub group_deliverable_selection_id: i32,
    pub group_id: i32,
    pub group_deliverable_id: i32,
    pub group_deliverable_name: String,
    pub component_implementation_details: Vec<ComponentImplementationDetail>,
}

#[utoipa::path(
    get,
    path = "/v1/students/group-deliverable-selections/{group_id}",
    responses(
        (status = 200, description = "Deliverable selection found", body = GroupDeliverableSelectionResponse),
        (status = 404, description = "No deliverable selected yet or group not found", body = JsonError),
        (status = 500, description = "Internal server error", body = JsonError)
    ),
    security(("StudentAuth" = [])),
    tag = "Group Deliverable Selections",
)]
/// Get the deliverable selection for a group
#[actix_web_grants::protect("ROLE_STUDENT")]
pub(in crate::api::v1) async fn get_group_deliverable_selection(
    path: Path<i32>, data: Data<AppData>,
) -> Result<HttpResponse, JsonError> {
    let group_id = path.into_inner();

    // Get the selection
    let selection_state =
        group_deliverable_selections_repository::get_by_group_id(&data.db, group_id)
            .await
            .map_err(|e| {
                error_with_log_id(
                    format!("Database error fetching selection: {}", e),
                    "Database error",
                    StatusCode::INTERNAL_SERVER_ERROR,
                    log::Level::Error,
                )
            })?
            .ok_or_else(|| {
                error_with_log_id(
                    format!("No deliverable selection found for group {}", group_id),
                    "No deliverable selected yet",
                    StatusCode::NOT_FOUND,
                    log::Level::Info,
                )
            })?;

    let selection = DbState::into_inner(selection_state);

    // Get the deliverable name
    let deliverable_state =
        group_deliverables_repository::get_by_id(&data.db, selection.group_deliverable_id)
            .await
            .map_err(|e| {
                error_with_log_id(
                    format!("Database error fetching deliverable: {}", e),
                    "Database error",
                    StatusCode::INTERNAL_SERVER_ERROR,
                    log::Level::Error,
                )
            })?
            .ok_or_else(|| {
                error_with_log_id(
                    format!(
                        "Deliverable {} not found for selection",
                        selection.group_deliverable_id
                    ),
                    "Deliverable not found",
                    StatusCode::NOT_FOUND,
                    log::Level::Error,
                )
            })?;

    let deliverable = DbState::into_inner(deliverable_state);

    // Get component implementation details
    let details_states = group_component_implementation_details_repository::get_by_selection_id(
        &data.db,
        selection.group_deliverable_selection_id,
    )
    .await
    .map_err(|e| {
        error_with_log_id(
            format!("Database error fetching implementation details: {}", e),
            "Database error",
            StatusCode::INTERNAL_SERVER_ERROR,
            log::Level::Error,
        )
    })?;

    let mut component_implementation_details = Vec::new();

    for detail_state in details_states {
        let detail = DbState::into_inner(detail_state);

        // Get the component name
        let component_state = group_deliverable_components_repository::get_by_id(
            &data.db,
            detail.group_deliverable_component_id,
        )
        .await
        .map_err(|e| {
            error_with_log_id(
                format!("Database error fetching component: {}", e),
                "Database error",
                StatusCode::INTERNAL_SERVER_ERROR,
                log::Level::Error,
            )
        })?;

        let component_name = if let Some(component_state) = component_state {
            let component = DbState::into_inner(component_state);
            component.name
        } else {
            format!(
                "Unknown Component {}",
                detail.group_deliverable_component_id
            )
        };

        component_implementation_details.push(ComponentImplementationDetail {
            id: detail.id,
            group_deliverable_component_id: detail.group_deliverable_component_id,
            component_name,
            markdown_description: detail.markdown_description,
            repository_link: detail.repository_link,
            created_at: detail.created_at,
            updated_at: detail.updated_at,
        });
    }

    Ok(HttpResponse::Ok().json(GroupDeliverableSelectionResponse {
        group_deliverable_selection_id: selection.group_deliverable_selection_id,
        group_id: selection.group_id,
        group_deliverable_id: selection.group_deliverable_id,
        group_deliverable_name: deliverable.name,
        component_implementation_details,
    }))
}
