use crate::app_data::AppData;
use crate::common::json_error::{error_with_log_id_and_payload, JsonError, ToJsonError};
use crate::database::repositories::group_deliverable_components_repository;
use crate::models::group_deliverable_component::GroupDeliverableComponent;
use actix_web::http::StatusCode;
use actix_web::web::{Data, Json};
use actix_web::HttpResponse;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub(crate) struct CreateGroupComponentScheme {
    #[schema(example = "1")]
    pub project_id: i32,
    #[schema(example = "Robot")]
    pub name: String,
    #[schema(example = "true")]
    pub sellable: bool,
}

#[derive(Debug, Serialize, ToSchema)]
pub(crate) struct CreateGroupComponentResponse {
    #[schema(example = "123")]
    pub group_deliverable_component_id: i32,
    #[schema(example = "1")]
    pub project_id: i32,
    #[schema(example = "Robot")]
    pub name: String,
    #[schema(example = "true")]
    pub sellable: bool,
}

#[utoipa::path(
    post,
    path = "/v1/admins/group-deliverable-components",
    request_body = CreateGroupComponentScheme,
    responses(
        (status = 200, description = "Group deliverable component created successfully", body = CreateGroupComponentResponse),
        (status = 400, description = "Invalid data in request", body = JsonError),
        (status = 401, description = "Authentication required", body = JsonError),
        (status = 409, description = "Deliverable component with this name already exists for the project", body = JsonError),
        (status = 500, description = "Internal server error occurred", body = JsonError)
    ),
    security(("AdminAuth" = [])),
    tag = "Group deliverable components management",
)]
/// Creates a new group component.
///
/// This endpoint allows authenticated admins to create a new group component for a specific project.
#[actix_web_grants::protect(any("ROLE_ADMIN_ROOT", "ROLE_ADMIN_PROFESSOR"))]
pub(super) async fn create_group_component_handler(
    body: Json<CreateGroupComponentScheme>, data: Data<AppData>,
) -> Result<HttpResponse, JsonError> {
    // Check if component with this name already exists for the project
    let exists = group_deliverable_components_repository::check_name_exists(
        &data.db,
        body.project_id,
        &body.name,
    )
    .await
    .map_err(|e| {
        error_with_log_id_and_payload(
            format!("unable to check existing component: {}", e),
            "Failed to create component",
            StatusCode::INTERNAL_SERVER_ERROR,
            log::Level::Error,
            &body,
        )
    })?;

    if exists {
        return Err("Component with this name already exists for the project"
            .to_json_error(StatusCode::CONFLICT));
    }

    let group_deliverable_component = GroupDeliverableComponent {
        group_deliverable_component_id: 0,
        project_id: body.project_id,
        name: body.name.clone(),
        sellable: body.sellable,
    };

    let state =
        group_deliverable_components_repository::create(&data.db, group_deliverable_component)
            .await
            .map_err(|e| {
                error_with_log_id_and_payload(
                    format!("unable to create group component: {}", e),
                    "Failed to create component",
                    StatusCode::INTERNAL_SERVER_ERROR,
                    log::Level::Error,
                    &body,
                )
            })?;

    Ok(HttpResponse::Ok().json(CreateGroupComponentResponse {
        group_deliverable_component_id: state.group_deliverable_component_id,
        project_id: body.project_id,
        name: body.name.clone(),
        sellable: body.sellable,
    }))
}
