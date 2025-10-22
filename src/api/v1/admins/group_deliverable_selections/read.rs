use crate::app_data::AppData;
use crate::common::json_error::{error_with_log_id, JsonError};
use crate::database::repositories::{
    group_component_implementation_details_repository, group_deliverable_components_repository,
    group_deliverable_selections_repository, group_deliverables_repository, groups_repository,
    projects_repository,
};
use crate::jwt::get_user::LoggedUser;
use actix_web::http::StatusCode;
use actix_web::web::{Data, Path};
use actix_web::{HttpMessage, HttpRequest, HttpResponse};
use serde::Serialize;
use utoipa::ToSchema;
use welds::state::DbState;

#[derive(Debug, Serialize, ToSchema)]
pub(crate) struct GroupDeliverableSelectionsResponse {
    pub project_id: i32,
    pub project_name: String,
    pub selections: Vec<GroupSelectionInfo>,
}

#[derive(Debug, Serialize, ToSchema)]
pub(crate) struct ComponentImplementationDetailInfo {
    pub id: i32,
    pub group_deliverable_component_id: i32,
    pub component_name: String,
    pub markdown_description: String,
    pub repository_link: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Serialize, ToSchema)]
pub(crate) struct GroupSelectionInfo {
    pub group_deliverable_selection_id: i32,
    pub group_id: i32,
    pub group_name: String,
    pub group_deliverable_id: i32,
    pub group_deliverable_name: String,
    pub component_implementation_details: Vec<ComponentImplementationDetailInfo>,
}

#[utoipa::path(
    get,
    path = "/v1/admins/projects/{project_id}/group-deliverable-selections",
    responses(
        (status = 200, description = "List of group deliverable selections", body = GroupDeliverableSelectionsResponse),
        (status = 401, description = "Authentication required", body = JsonError),
        (status = 404, description = "Project not found", body = JsonError),
        (status = 500, description = "Internal server error", body = JsonError)
    ),
    security(("AdminAuth" = [])),
    tag = "Admin Group Deliverable Selections",
)]
/// List all group deliverable selections for a project
///
/// This endpoint allows admins to view all group deliverable selections for a specific project,
/// including which deliverables each group has chosen and their submission details.
#[actix_web_grants::protect(any(
    "ROLE_ADMIN_ROOT",
    "ROLE_ADMIN_PROFESSOR",
    "ROLE_ADMIN_COORDINATOR"
))]
pub(super) async fn get_group_deliverable_selections(
    req: HttpRequest, path: Path<i32>, data: Data<AppData>,
) -> Result<HttpResponse, JsonError> {
    let _admin = match req.extensions().get_admin() {
        Ok(admin) => admin,
        Err(_) => {
            return Err(error_with_log_id(
                "entered a protected route without an admin loaded in the request",
                "Authentication error",
                StatusCode::INTERNAL_SERVER_ERROR,
                log::Level::Error,
            ));
        }
    };

    let project_id = path.into_inner();

    // Verify the project exists
    let project_state = projects_repository::get_by_id(&data.db, project_id)
        .await
        .map_err(|e| {
            error_with_log_id(
                format!("unable to fetch project {}: {}", project_id, e),
                "Database error",
                StatusCode::INTERNAL_SERVER_ERROR,
                log::Level::Error,
            )
        })?;

    let project = match project_state {
        Some(state) => DbState::into_inner(state),
        None => {
            return Err(error_with_log_id(
                format!("project {} not found", project_id),
                "Project not found",
                StatusCode::NOT_FOUND,
                log::Level::Warn,
            ));
        }
    };

    // Get all groups for this project
    let groups = groups_repository::get_by_project_id(&data.db, project_id)
        .await
        .map_err(|e| {
            error_with_log_id(
                format!("unable to fetch groups for project {}: {}", project_id, e),
                "Database error",
                StatusCode::INTERNAL_SERVER_ERROR,
                log::Level::Error,
            )
        })?;

    let mut selections = Vec::new();

    for group_state in groups {
        let group = DbState::into_inner(group_state);

        // Get the group's deliverable selection
        let selection_state =
            group_deliverable_selections_repository::get_by_group_id(&data.db, group.group_id)
                .await
                .map_err(|e| {
                    error_with_log_id(
                        format!(
                            "unable to fetch deliverable selection for group {}: {}",
                            group.group_id, e
                        ),
                        "Database error",
                        StatusCode::INTERNAL_SERVER_ERROR,
                        log::Level::Error,
                    )
                })?;

        // Only include groups that have made a selection
        if let Some(selection_state) = selection_state {
            let selection = DbState::into_inner(selection_state);

            // Get the deliverable details
            let deliverable_state =
                group_deliverables_repository::get_by_id(&data.db, selection.group_deliverable_id)
                    .await
                    .map_err(|e| {
                        error_with_log_id(
                            format!(
                                "unable to fetch deliverable {}: {}",
                                selection.group_deliverable_id, e
                            ),
                            "Database error",
                            StatusCode::INTERNAL_SERVER_ERROR,
                            log::Level::Error,
                        )
                    })?;

            let deliverable = match deliverable_state {
                Some(state) => DbState::into_inner(state),
                None => {
                    // Log warning but continue processing other selections
                    log::warn!(
                        "Deliverable {} not found for selection {}",
                        selection.group_deliverable_id,
                        selection.group_deliverable_selection_id
                    );
                    continue;
                }
            };

            // Get component implementation details for this selection
            let details_states =
                group_component_implementation_details_repository::get_by_selection_id(
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

                component_implementation_details.push(ComponentImplementationDetailInfo {
                    id: detail.id,
                    group_deliverable_component_id: detail.group_deliverable_component_id,
                    component_name,
                    markdown_description: detail.markdown_description,
                    repository_link: detail.repository_link,
                    created_at: detail.created_at,
                    updated_at: detail.updated_at,
                });
            }

            selections.push(GroupSelectionInfo {
                group_deliverable_selection_id: selection.group_deliverable_selection_id,
                group_id: group.group_id,
                group_name: group.name,
                group_deliverable_id: selection.group_deliverable_id,
                group_deliverable_name: deliverable.name,
                component_implementation_details,
            });
        }
    }

    Ok(HttpResponse::Ok().json(GroupDeliverableSelectionsResponse {
        project_id,
        project_name: project.name,
        selections,
    }))
}
