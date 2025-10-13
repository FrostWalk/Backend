use crate::app_data::AppData;
use crate::common::json_error::{error_with_log_id, JsonError};
use crate::database::repositories::group_deliverable_selections_repository;
use crate::models::group_deliverable::GroupDeliverable;
use actix_web::http::StatusCode;
use actix_web::web::{Data, Path};
use actix_web::HttpResponse;
use serde::Serialize;
use utoipa::ToSchema;
use welds::state::DbState;

#[derive(Debug, Serialize, ToSchema)]
pub(crate) struct GroupDeliverableSelectionResponse {
    pub group_deliverable_selection_id: i32,
    pub group_id: i32,
    pub group_deliverable_id: i32,
    pub group_deliverable_name: String,
    pub link: String,
    pub markdown_text: String,
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
pub(in crate::api::v1) async fn get_group_deliverable_selection(
    group_id: Path<i32>, data: Data<AppData>,
) -> Result<HttpResponse, JsonError> {
    let group_id = group_id.into_inner();

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
    let mut deliverable_rows = GroupDeliverable::where_col(|gd| {
        gd.group_deliverable_id
            .equal(selection.group_deliverable_id)
    })
    .run(&data.db)
    .await
    .map_err(|e| {
        error_with_log_id(
            format!("Database error fetching deliverable: {}", e),
            "Database error",
            StatusCode::INTERNAL_SERVER_ERROR,
            log::Level::Error,
        )
    })?;

    let deliverable_state = deliverable_rows.pop().ok_or_else(|| {
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

    Ok(HttpResponse::Ok().json(GroupDeliverableSelectionResponse {
        group_deliverable_selection_id: selection.group_deliverable_selection_id,
        group_id: selection.group_id,
        group_deliverable_id: selection.group_deliverable_id,
        group_deliverable_name: deliverable.name,
        link: selection.link,
        markdown_text: selection.markdown_text,
    }))
}
