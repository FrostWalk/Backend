use crate::app_data::AppData;
use crate::common::json_error::{error_with_log_id, error_with_log_id_and_payload, JsonError};
use crate::database::repositories::{
    group_component_implementation_details_repository, group_deliverable_selections_repository,
    group_deliverables_components_repository, groups_repository,
};
use crate::jwt::get_user::LoggedUser;
use actix_web::http::StatusCode;
use actix_web::web::{Data, Json, Path};
use actix_web::{HttpMessage, HttpRequest, HttpResponse};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub(crate) struct CreateComponentImplementationDetailRequest {
    #[schema(example = 5)]
    pub group_deliverable_component_id: i32,
    #[schema(example = "# Component Description\n\nThis component handles...")]
    pub markdown_description: String,
    #[schema(example = "https://github.com/group1/component")]
    pub repository_link: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub(crate) struct CreateComponentImplementationDetailResponse {
    pub id: i32,
    pub message: String,
}

#[utoipa::path(
    post,
    path = "/v1/students/group-component-implementation-details/{group_id}",
    request_body = CreateComponentImplementationDetailRequest,
    responses(
        (status = 201, description = "Component implementation detail created successfully", body = CreateComponentImplementationDetailResponse),
        (status = 400, description = "Invalid request", body = JsonError),
        (status = 403, description = "Not authorized - must be group leader", body = JsonError),
        (status = 404, description = "Group, selection, or component not found", body = JsonError),
        (status = 409, description = "Implementation details already exist for this component", body = JsonError),
        (status = 500, description = "Internal server error", body = JsonError)
    ),
    security(("StudentAuth" = [])),
    tag = "Group Component Implementation Details",
)]
/// Create implementation details for a single component (Group Leaders only)
#[actix_web_grants::protect("ROLE_STUDENT")]
pub(in crate::api::v1) async fn create_component_implementation_detail(
    req: HttpRequest, path: Path<i32>, body: Json<CreateComponentImplementationDetailRequest>,
    data: Data<AppData>,
) -> Result<HttpResponse, JsonError> {
    let group_id = path.into_inner();

    // Get the logged-in user
    let user = req.extensions().get_student().map_err(|_| {
        error_with_log_id(
            "entered a protected route without a user loaded in the request",
            "Authentication error",
            StatusCode::INTERNAL_SERVER_ERROR,
            log::Level::Error,
        )
    })?;

    // Validate input
    if body.markdown_description.trim().is_empty() {
        return Err(JsonError::new(
            "Markdown description field is mandatory",
            StatusCode::BAD_REQUEST,
        ));
    }

    if body.repository_link.trim().is_empty() {
        return Err(JsonError::new(
            "Repository link field is mandatory",
            StatusCode::BAD_REQUEST,
        ));
    }

    // 1. Verify the user is a Group Leader of the group
    let is_leader = groups_repository::is_group_leader(&data.db, user.student_id, group_id)
        .await
        .map_err(|e| {
            error_with_log_id(
                format!("Database error checking group leader status: {}", e),
                "Database error",
                StatusCode::INTERNAL_SERVER_ERROR,
                log::Level::Error,
            )
        })?;

    if !is_leader {
        return Err(error_with_log_id(
            format!(
                "Student {} is not a group leader of group {}",
                user.student_id, group_id
            ),
            "Only group leaders can create component implementation details",
            StatusCode::FORBIDDEN,
            log::Level::Warn,
        ));
    }

    // 2. Verify the group has selected a deliverable
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

    let selection = welds::state::DbState::into_inner(selection_state);

    // 3. Verify the component is part of the selected deliverable
    let component_exists = group_deliverables_components_repository::is_component_in_deliverable(
        &data.db,
        selection.group_deliverable_id,
        body.group_deliverable_component_id,
    )
    .await
    .map_err(|e| {
        error_with_log_id(
            format!("Database error checking component: {}", e),
            "Database error",
            StatusCode::INTERNAL_SERVER_ERROR,
            log::Level::Error,
        )
    })?;

    if !component_exists {
        return Err(error_with_log_id(
            format!(
                "Component {} is not part of deliverable {}",
                body.group_deliverable_component_id, selection.group_deliverable_id
            ),
            "Component is not part of the selected deliverable",
            StatusCode::NOT_FOUND,
            log::Level::Warn,
        ));
    }

    // 4. Verify implementation details don't already exist for this component
    let exists = group_component_implementation_details_repository::exists(
        &data.db,
        selection.group_deliverable_selection_id,
        body.group_deliverable_component_id,
    )
    .await
    .map_err(|e| {
        error_with_log_id(
            format!("Database error checking existing details: {}", e),
            "Database error",
            StatusCode::INTERNAL_SERVER_ERROR,
            log::Level::Error,
        )
    })?;

    if exists {
        return Err(error_with_log_id_and_payload(
            format!(
                "Implementation details already exist for component {}",
                body.group_deliverable_component_id
            ),
            "Implementation details already exist for this component",
            StatusCode::CONFLICT,
            log::Level::Warn,
            &body,
        ));
    }

    // 5. Create the implementation detail
    let detail_state = group_component_implementation_details_repository::create(
        &data.db,
        selection.group_deliverable_selection_id,
        body.group_deliverable_component_id,
        body.markdown_description.clone(),
        body.repository_link.clone(),
    )
    .await
    .map_err(|e| {
        error_with_log_id_and_payload(
            format!("Failed to create component implementation detail: {}", e),
            "Failed to create implementation detail",
            StatusCode::INTERNAL_SERVER_ERROR,
            log::Level::Error,
            &body,
        )
    })?;

    let detail = welds::state::DbState::into_inner(detail_state);

    Ok(
        HttpResponse::Created().json(CreateComponentImplementationDetailResponse {
            id: detail.id,
            message: "Component implementation detail created successfully".to_string(),
        }),
    )
}
