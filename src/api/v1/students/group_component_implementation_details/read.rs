use crate::app_data::AppData;
use crate::common::json_error::{error_with_log_id, JsonError};
use crate::database::repositories::{
    group_component_implementation_details_repository, group_deliverable_selections_repository,
};
use crate::models::group_deliverable_component::GroupDeliverableComponent;
use actix_web::http::StatusCode;
use actix_web::web::{Data, Path};
use actix_web::HttpResponse;
use serde::Serialize;
use utoipa::ToSchema;
use welds::state::DbState;

#[derive(Debug, Serialize, ToSchema)]
pub(crate) struct ComponentImplementationDetailResponse {
    pub id: i32,
    pub group_deliverable_component_id: i32,
    pub component_name: String,
    pub markdown_description: String,
    pub repository_link: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Serialize, ToSchema)]
pub(crate) struct GetComponentImplementationDetailsResponse {
    pub details: Vec<ComponentImplementationDetailResponse>,
}

#[utoipa::path(
    get,
    path = "/v1/students/group-component-implementation-details/{group_id}",
    responses(
        (status = 200, description = "Component implementation details found", body = GetComponentImplementationDetailsResponse),
        (status = 404, description = "Group or selection not found", body = JsonError),
        (status = 500, description = "Internal server error", body = JsonError)
    ),
    security(("StudentAuth" = [])),
    tag = "Group Component Implementation Details",
)]
/// Get all implementation details for a group's selection
pub(in crate::api::v1) async fn get_component_implementation_details(
    path: Path<i32>, 
    data: Data<AppData>,
) -> Result<HttpResponse, JsonError> {
    let group_id = path.into_inner();

    // 1. Verify the group has selected a deliverable
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
                    "Group must select a deliverable first",
                    StatusCode::NOT_FOUND,
                    log::Level::Warn,
                )
            })?;

    let selection = DbState::into_inner(selection_state);

    // 2. Get all implementation details for this selection
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

    let mut details = Vec::new();

    for detail_state in details_states {
        let detail = DbState::into_inner(detail_state);

        // Get the component name
        let mut component_rows = GroupDeliverableComponent::where_col(|gdc| {
            gdc.group_deliverable_component_id
                .equal(detail.group_deliverable_component_id)
        })
        .run(&data.db)
        .await
        .map_err(|e| {
            error_with_log_id(
                format!("Database error fetching component: {}", e),
                "Database error",
                StatusCode::INTERNAL_SERVER_ERROR,
                log::Level::Error,
            )
        })?;

        let component_name = if let Some(component_state) = component_rows.pop() {
            let component = DbState::into_inner(component_state);
            component.name
        } else {
            format!(
                "Unknown Component {}",
                detail.group_deliverable_component_id
            )
        };

        details.push(ComponentImplementationDetailResponse {
            id: detail.id,
            group_deliverable_component_id: detail.group_deliverable_component_id,
            component_name,
            markdown_description: detail.markdown_description,
            repository_link: detail.repository_link,
            created_at: detail.created_at,
            updated_at: detail.updated_at,
        });
    }

    Ok(HttpResponse::Ok().json(GetComponentImplementationDetailsResponse { details }))
}
