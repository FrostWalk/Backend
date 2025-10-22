use crate::app_data::AppData;
use crate::common::json_error::{error_with_log_id, JsonError};
use crate::database::repositories::{
    group_component_implementation_details_repository, group_deliverable_selections_repository,
    group_deliverables_repository, groups_repository, projects_repository,
    student_deliverable_selections_repository, students_repository,
};
use crate::jwt::get_user::LoggedUser;
use crate::models::group_deliverable_component::GroupDeliverableComponent;
use crate::models::student_deliverable::StudentDeliverable;
use crate::models::student_deliverable_component::StudentDeliverableComponent;
use crate::models::student_deliverables_component::StudentDeliverablesComponent;
use actix_web::http::StatusCode;
use actix_web::web::{Data, Path};
use actix_web::{HttpMessage, HttpRequest, HttpResponse};
use serde::Serialize;
use utoipa::ToSchema;
use welds::state::DbState;

#[derive(Debug, Serialize, ToSchema)]
pub(crate) struct GroupDetailsResponse {
    pub group_id: i32,
    pub name: String,
    pub project_id: i32,
    pub project_name: String,
    pub members: Vec<GroupMemberDetail>,
    pub deliverable_selection: Option<GroupDeliverableDetail>,
}

#[derive(Debug, Serialize, ToSchema)]
pub(crate) struct GroupMemberDetail {
    pub student_id: i32,
    pub first_name: String,
    pub last_name: String,
    pub email: String,
    pub university_id: i32,
    pub role: String,
    pub student_deliverable_selection: Option<StudentDeliverableSelectionDetail>,
}

#[derive(Debug, Serialize, ToSchema)]
pub(crate) struct StudentDeliverableSelectionDetail {
    pub student_deliverable_id: i32,
    pub student_deliverable_name: String,
    pub components: Vec<ComponentDetail>,
}

#[derive(Debug, Serialize, ToSchema)]
pub(crate) struct ComponentDetail {
    pub student_deliverable_component_id: i32,
    pub name: String,
    pub project_id: i32,
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
pub(crate) struct GroupDeliverableDetail {
    pub group_deliverable_id: i32,
    pub name: String,
    pub component_implementation_details: Vec<ComponentImplementationDetailInfo>,
}

#[utoipa::path(
    get,
    path = "/v1/admins/groups/{group_id}",
    responses(
        (status = 200, description = "Detailed group information", body = GroupDetailsResponse),
        (status = 401, description = "Authentication required", body = JsonError),
        (status = 404, description = "Group not found", body = JsonError),
        (status = 500, description = "Internal server error", body = JsonError)
    ),
    security(("AdminAuth" = [])),
    tag = "Admin Groups management",
)]
/// Get detailed information about a specific group
///
/// This endpoint allows admins to view comprehensive details about a group including:
/// - All members with their roles
/// - The group's deliverable selection (if any)
/// - Individual student deliverable selections for group members
#[actix_web_grants::protect(any(
    "ROLE_ADMIN_ROOT",
    "ROLE_ADMIN_PROFESSOR",
    "ROLE_ADMIN_COORDINATOR"
))]
pub(super) async fn get_group_details(
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

    let group_id = path.into_inner();

    // Get the group details
    let group_state = groups_repository::get_by_id(&data.db, group_id)
        .await
        .map_err(|e| {
            error_with_log_id(
                format!("unable to fetch group {}: {}", group_id, e),
                "Database error",
                StatusCode::INTERNAL_SERVER_ERROR,
                log::Level::Error,
            )
        })?;

    let group = match group_state {
        Some(state) => DbState::into_inner(state),
        None => {
            return Err(error_with_log_id(
                format!("group {} not found", group_id),
                "Group not found",
                StatusCode::NOT_FOUND,
                log::Level::Warn,
            ));
        }
    };

    // Get project details
    let project_state = projects_repository::get_by_id(&data.db, group.project_id)
        .await
        .map_err(|e| {
            error_with_log_id(
                format!("unable to fetch project {}: {}", group.project_id, e),
                "Database error",
                StatusCode::INTERNAL_SERVER_ERROR,
                log::Level::Error,
            )
        })?;

    let project = match project_state {
        Some(state) => DbState::into_inner(state),
        None => {
            return Err(error_with_log_id(
                format!("project {} not found", group.project_id),
                "Project not found",
                StatusCode::NOT_FOUND,
                log::Level::Warn,
            ));
        }
    };

    // Get all group members
    let group_members_states = groups_repository::get_group_members(&data.db, group_id)
        .await
        .map_err(|e| {
            error_with_log_id(
                format!(
                    "unable to fetch group members for group {}: {}",
                    group_id, e
                ),
                "Database error",
                StatusCode::INTERNAL_SERVER_ERROR,
                log::Level::Error,
            )
        })?;

    let mut members = Vec::new();

    for member_state in group_members_states {
        let member = DbState::into_inner(member_state);

        // Get student details
        let student_state = students_repository::get_by_id(&data.db, member.student_id)
            .await
            .map_err(|e| {
                error_with_log_id(
                    format!("unable to fetch student {}: {}", member.student_id, e),
                    "Database error",
                    StatusCode::INTERNAL_SERVER_ERROR,
                    log::Level::Error,
                )
            })?;

        if let Some(student_state) = student_state {
            let student = DbState::into_inner(student_state);
            let role = match member.student_role_id {
                1 => "Group Leader".to_string(),
                2 => "Member".to_string(),
                _ => "Unknown".to_string(),
            };

            // Get student's deliverable selection for this project
            let student_deliverable_selection =
                match student_deliverable_selections_repository::get_by_student_and_project(
                    &data.db,
                    student.student_id,
                    group.project_id,
                )
                .await
                {
                    Ok(Some(selection_state)) => {
                        let selection = DbState::into_inner(selection_state);

                        // Get the deliverable details
                        let deliverable_state = StudentDeliverable::find_by_id(
                            &data.db,
                            selection.student_deliverable_id,
                        )
                        .await
                        .map_err(|e| {
                            error_with_log_id(
                                format!(
                                    "unable to fetch student deliverable {}: {}",
                                    selection.student_deliverable_id, e
                                ),
                                "Database error",
                                StatusCode::INTERNAL_SERVER_ERROR,
                                log::Level::Error,
                            )
                        })?;

                        if let Some(deliverable_state) = deliverable_state {
                            let deliverable = DbState::into_inner(deliverable_state);

                            // Get components for this deliverable
                            let components_relations =
                                StudentDeliverablesComponent::where_col(|sdc| {
                                    sdc.student_deliverable_id
                                        .equal(selection.student_deliverable_id)
                                })
                                .run(&data.db)
                                .await
                                .map_err(|e| {
                                    error_with_log_id(
                                        format!(
                                            "unable to fetch components for deliverable {}: {}",
                                            selection.student_deliverable_id, e
                                        ),
                                        "Database error",
                                        StatusCode::INTERNAL_SERVER_ERROR,
                                        log::Level::Error,
                                    )
                                })?;

                            let mut components = Vec::new();
                            for relation_state in components_relations {
                                let relation = DbState::into_inner(relation_state);

                                // Get component details
                                let component_state = StudentDeliverableComponent::find_by_id(
                                    &data.db,
                                    relation.student_deliverable_component_id,
                                )
                                .await
                                .map_err(|e| {
                                    error_with_log_id(
                                        format!(
                                            "unable to fetch component {}: {}",
                                            relation.student_deliverable_component_id, e
                                        ),
                                        "Database error",
                                        StatusCode::INTERNAL_SERVER_ERROR,
                                        log::Level::Error,
                                    )
                                })?;

                                if let Some(component_state) = component_state {
                                    let component = DbState::into_inner(component_state);
                                    components.push(ComponentDetail {
                                        student_deliverable_component_id: component
                                            .student_deliverable_component_id,
                                        name: component.name,
                                        project_id: component.project_id,
                                    });
                                }
                            }

                            Some(StudentDeliverableSelectionDetail {
                                student_deliverable_id: selection.student_deliverable_id,
                                student_deliverable_name: deliverable.name,
                                components,
                            })
                        } else {
                            None
                        }
                    }
                    Ok(None) => None,
                    Err(e) => {
                        log::warn!(
                            "Failed to fetch deliverable selection for student {}: {}",
                            student.student_id,
                            e
                        );
                        None
                    }
                };

            members.push(GroupMemberDetail {
                student_id: student.student_id,
                first_name: student.first_name,
                last_name: student.last_name,
                email: student.email,
                university_id: student.university_id,
                role,
                student_deliverable_selection,
            });
        }
    }

    // Get the group's deliverable selection
    let deliverable_selection =
        group_deliverable_selections_repository::get_by_group_id(&data.db, group_id)
            .await
            .map_err(|e| {
                error_with_log_id(
                    format!(
                        "unable to fetch deliverable selection for group {}: {}",
                        group_id, e
                    ),
                    "Database error",
                    StatusCode::INTERNAL_SERVER_ERROR,
                    log::Level::Error,
                )
            })?;

    let deliverable_detail = if let Some(selection_state) = deliverable_selection {
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

        if let Some(deliverable_state) = deliverable_state {
            let deliverable = DbState::into_inner(deliverable_state);

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

            Some(GroupDeliverableDetail {
                group_deliverable_id: selection.group_deliverable_id,
                name: deliverable.name,
                component_implementation_details,
            })
        } else {
            None
        }
    } else {
        None
    };

    Ok(HttpResponse::Ok().json(GroupDetailsResponse {
        group_id,
        name: group.name,
        project_id: group.project_id,
        project_name: project.name,
        members,
        deliverable_selection: deliverable_detail,
    }))
}
