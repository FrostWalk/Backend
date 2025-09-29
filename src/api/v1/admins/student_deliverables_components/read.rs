use crate::app_data::AppData;
use crate::common::json_error::{error_with_log_id, JsonError, ToJsonError};
use crate::models::student_deliverable::StudentDeliverable;
use crate::models::student_deliverable_component::StudentDeliverableComponent;
use crate::models::student_deliverables_component::StudentDeliverablesComponent;
use actix_web::http::StatusCode;
use actix_web::web::Data;
use actix_web::{web, HttpResponse};
use serde::Serialize;
use utoipa::ToSchema;
use welds::state::DbState;

#[derive(Debug, Serialize, ToSchema)]
pub(crate) struct StudentDeliverableComponentResponse {
    #[schema(example = "123")]
    pub id: i32,
    #[schema(example = "1")]
    pub student_deliverable_id: i32,
    #[schema(example = "2")]
    pub student_deliverable_component_id: i32,
    #[schema(example = "5")]
    pub quantity: i32,
    #[schema(example = "Resistor")]
    pub component_name: String,
    #[schema(example = "10k")]
    pub deliverable_name: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub(crate) struct GetComponentsForDeliverableResponse {
    pub components: Vec<StudentDeliverableComponentResponse>,
}

#[derive(Debug, Serialize, ToSchema)]
pub(crate) struct GetDeliverablesForComponentResponse {
    pub deliverables: Vec<StudentDeliverableComponentResponse>,
}

#[utoipa::path(
    get,
    path = "/v1/admins/student-deliverables-components/components/{deliverable_id}",
    responses(
        (status = 200, description = "Found components for student deliverable", body = GetComponentsForDeliverableResponse),
        (status = 404, description = "Student deliverable not found", body = JsonError),
        (status = 500, description = "Internal server error occurred", body = JsonError)
    ),
    security(("AdminAuth" = [])),
    tag = "Student deliverables-components management",
)]
/// Get all components for a specific student deliverable.
///
/// Returns all components associated with the specified student deliverable along with their quantities.
pub(super) async fn get_components_for_deliverable_handler(
    path: web::Path<i32>, data: Data<AppData>,
) -> Result<HttpResponse, JsonError> {
    let deliverable_id = path.into_inner();

    // Verify the student deliverable exists
    let deliverable_exists =
        StudentDeliverable::where_col(|sp| sp.student_deliverable_id.equal(deliverable_id))
            .run(&data.db)
            .await
            .map_err(|e| {
                error_with_log_id(
                    format!("unable to check if student deliverable exists: {}", e),
                    "Failed to retrieve components",
                    StatusCode::INTERNAL_SERVER_ERROR,
                    log::Level::Error,
                )
            })?;

    if deliverable_exists.is_empty() {
        return Err("Student deliverable not found".to_json_error(StatusCode::NOT_FOUND));
    }

    // Get all relationships for this deliverable
    let relationships = StudentDeliverablesComponent::where_col(|spc| {
        spc.student_deliverable_id.equal(deliverable_id)
    })
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
        let mut component_rows = StudentDeliverableComponent::where_col(|sc| {
            sc.student_deliverable_component_id
                .equal(relationship_data.student_deliverable_component_id)
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
        let mut deliverable_rows = StudentDeliverable::where_col(|sp| {
            sp.student_deliverable_id
                .equal(relationship_data.student_deliverable_id)
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

        components.push(StudentDeliverableComponentResponse {
            id: relationship_data.id,
            student_deliverable_id: relationship_data.student_deliverable_id,
            student_deliverable_component_id: relationship_data.student_deliverable_component_id,
            quantity: relationship_data.quantity,
            component_name: component.name,
            deliverable_name: deliverable.name,
        });
    }

    Ok(HttpResponse::Ok().json(GetComponentsForDeliverableResponse { components }))
}

#[utoipa::path(
    get,
    path = "/v1/admins/student-deliverables-components/deliverables/{component_id}",
    responses(
        (status = 200, description = "Found deliverables for student component", body = GetDeliverablesForComponentResponse),
        (status = 404, description = "Student deliverable component not found", body = JsonError),
        (status = 500, description = "Internal server error occurred", body = JsonError)
    ),
    security(("AdminAuth" = [])),
    tag = "Student deliverables-components management",
)]
/// Get all deliverables that use a specific student component.
///
/// Returns all student deliverables that use the specified component along with their quantities.
pub(super) async fn get_deliverables_for_component_handler(
    path: web::Path<i32>, data: Data<AppData>,
) -> Result<HttpResponse, JsonError> {
    let component_id = path.into_inner();

    // Verify the student component exists
    let component_exists = StudentDeliverableComponent::where_col(|sc| {
        sc.student_deliverable_component_id.equal(component_id)
    })
    .run(&data.db)
    .await
    .map_err(|e| {
        error_with_log_id(
            format!("unable to check if student component exists: {}", e),
            "Failed to retrieve deliverables",
            StatusCode::INTERNAL_SERVER_ERROR,
            log::Level::Error,
        )
    })?;

    if component_exists.is_empty() {
        return Err("Student component not found".to_json_error(StatusCode::NOT_FOUND));
    }

    // Get all relationships for this component
    let relationships = StudentDeliverablesComponent::where_col(|spc| {
        spc.student_deliverable_component_id.equal(component_id)
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
        let mut component_rows = StudentDeliverableComponent::where_col(|sc| {
            sc.student_deliverable_component_id
                .equal(relationship_data.student_deliverable_component_id)
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
        let mut deliverable_rows = StudentDeliverable::where_col(|sp| {
            sp.student_deliverable_id
                .equal(relationship_data.student_deliverable_id)
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

        deliverables.push(StudentDeliverableComponentResponse {
            id: relationship_data.id,
            student_deliverable_id: relationship_data.student_deliverable_id,
            student_deliverable_component_id: relationship_data.student_deliverable_component_id,
            quantity: relationship_data.quantity,
            component_name: component.name,
            deliverable_name: deliverable.name,
        });
    }

    Ok(HttpResponse::Ok().json(GetDeliverablesForComponentResponse { deliverables }))
}
