use crate::app_data::AppData;
use crate::common::json_error::{error_with_log_id_and_payload, JsonError, ToJsonError};
use crate::models::group_deliverable::GroupDeliverable;
use actix_web::http::StatusCode;
use actix_web::web::{Data, Json};
use actix_web::HttpResponse;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use welds::state::DbState;

#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub(crate) struct CreateGroupDeliverableScheme {
    #[schema(example = "1")]
    pub project_id: i32,
    #[schema(example = "Motor")]
    pub name: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub(crate) struct CreateGroupDeliverableResponse {
    #[schema(example = "123")]
    pub group_deliverable_id: i32,
    #[schema(example = "1")]
    pub project_id: i32,
    #[schema(example = "Motor")]
    pub name: String,
}

#[utoipa::path(
    post,
    path = "/v1/admins/group-deliverables",
    request_body = CreateGroupDeliverableScheme,
    responses(
        (status = 200, description = "Group deliverable created successfully", body = CreateGroupDeliverableResponse),
        (status = 400, description = "Invalid data in request", body = JsonError),
        (status = 401, description = "Authentication required", body = JsonError),
        (status = 409, description = "Deliverable with this name already exists for the project", body = JsonError),
        (status = 500, description = "Internal server error occurred", body = JsonError)
    ),
    security(("AdminAuth" = [])),
    tag = "Group deliverables management",
)]
/// Creates a new group deliverable.
///
/// This endpoint allows authenticated admins to create a new group deliverable for a specific project.
pub(super) async fn create_group_deliverable_handler(
    req: Json<CreateGroupDeliverableScheme>, data: Data<AppData>,
) -> Result<HttpResponse, JsonError> {
    // Check if deliverable with this name already exists for the project
    let existing = GroupDeliverable::where_col(|gd| gd.project_id.equal(req.project_id))
        .where_col(|gd| gd.name.equal(&req.name))
        .run(&data.db)
        .await
        .map_err(|e| {
            error_with_log_id_and_payload(
                format!("unable to check existing group deliverable: {}", e),
                "Failed to create deliverable",
                StatusCode::INTERNAL_SERVER_ERROR,
                log::Level::Error,
                &req,
            )
        })?;

    if !existing.is_empty() {
        return Err("Deliverable with this name already exists for the project"
            .to_json_error(StatusCode::CONFLICT));
    }

    let mut state = DbState::new_uncreated(GroupDeliverable {
        group_deliverable_id: 0,
        project_id: req.project_id,
        name: req.name.clone(),
    });

    if let Err(e) = state.save(&data.db).await {
        return Err(error_with_log_id_and_payload(
            format!("unable to create group deliverable: {}", e),
            "Failed to create deliverable",
            StatusCode::INTERNAL_SERVER_ERROR,
            log::Level::Error,
            &req,
        ));
    }

    Ok(HttpResponse::Ok().json(CreateGroupDeliverableResponse {
        group_deliverable_id: state.group_deliverable_id,
        project_id: req.project_id,
        name: req.name.clone(),
    }))
}
