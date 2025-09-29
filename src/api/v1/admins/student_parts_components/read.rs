use crate::app_data::AppData;
use crate::common::json_error::{error_with_log_id, JsonError, ToJsonError};
use crate::models::student_parts_component::StudentPartsComponent;
use crate::models::student_part::StudentPart;
use crate::models::students_component::StudentsComponent;
use actix_web::http::StatusCode;
use actix_web::web::Data;
use actix_web::{web, HttpResponse};
use serde::Serialize;
use utoipa::ToSchema;
use welds::state::DbState;

#[derive(Debug, Serialize, ToSchema)]
pub(crate) struct StudentPartComponentResponse {
    #[schema(example = "123")]
    pub id: i32,
    #[schema(example = "1")]
    pub student_part_id: i32,
    #[schema(example = "2")]
    pub students_component_id: i32,
    #[schema(example = "5")]
    pub quantity: i32,
    #[schema(example = "Resistor")]
    pub component_name: String,
    #[schema(example = "10kΩ")]
    pub part_name: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub(crate) struct GetComponentsForPartResponse {
    pub components: Vec<StudentPartComponentResponse>,
}

#[derive(Debug, Serialize, ToSchema)]
pub(crate) struct GetPartsForComponentResponse {
    pub parts: Vec<StudentPartComponentResponse>,
}

#[utoipa::path(
    get,
    path = "/v1/admins/student-parts-components/components/{part_id}",
    responses(
        (status = 200, description = "Found components for student part", body = GetComponentsForPartResponse),
        (status = 404, description = "Student part not found", body = JsonError),
        (status = 500, description = "Internal server error occurred", body = JsonError)
    ),
    security(("AdminAuth" = [])),
    tag = "Student parts-components management",
)]
/// Get all components for a specific student part.
///
/// Returns all components associated with the specified student part along with their quantities.
pub(super) async fn get_components_for_part_handler(
    path: web::Path<i32>,
    data: Data<AppData>,
) -> Result<HttpResponse, JsonError> {
    let part_id = path.into_inner();

    // Verify the student part exists
    let part_exists = StudentPart::where_col(|sp| sp.student_part_id.equal(part_id))
        .run(&data.db)
        .await
        .map_err(|e| {
            error_with_log_id(
                format!("unable to check if student part exists: {}", e),
                "Failed to retrieve components",
                StatusCode::INTERNAL_SERVER_ERROR,
                log::Level::Error,
            )
        })?;

    if part_exists.is_empty() {
        return Err("Student part not found".to_json_error(StatusCode::NOT_FOUND));
    }

    // Get all relationships for this part
    let relationships = StudentPartsComponent::where_col(|spc| {
        spc.student_part_id.equal(part_id)
    })
    .run(&data.db)
    .await
    .map_err(|e| {
        error_with_log_id(
            format!("unable to retrieve components for part {}: {}", part_id, e),
            "Failed to retrieve components",
            StatusCode::INTERNAL_SERVER_ERROR,
            log::Level::Error,
        )
    })?;

    let mut components = Vec::new();

    for relationship in relationships {
        let relationship_data = DbState::into_inner(relationship);
        
        // Get component details
        let mut component_rows = StudentsComponent::where_col(|sc| {
            sc.students_component_id.equal(relationship_data.students_component_id)
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
            None => continue, // Skip if component not found
        };

        // Get part details
        let mut part_rows = StudentPart::where_col(|sp| {
            sp.student_part_id.equal(relationship_data.student_part_id)
        })
        .run(&data.db)
        .await
        .map_err(|e| {
            error_with_log_id(
                format!("unable to retrieve part details: {}", e),
                "Failed to retrieve components",
                StatusCode::INTERNAL_SERVER_ERROR,
                log::Level::Error,
            )
        })?;

        let part = match part_rows.pop() {
            Some(p) => DbState::into_inner(p),
            None => continue, // Skip if part not found
        };

        components.push(StudentPartComponentResponse {
            id: relationship_data.id,
            student_part_id: relationship_data.student_part_id,
            students_component_id: relationship_data.students_component_id,
            quantity: relationship_data.quantity,
            component_name: component.name,
            part_name: part.name,
        });
    }

    Ok(HttpResponse::Ok().json(GetComponentsForPartResponse { components }))
}

#[utoipa::path(
    get,
    path = "/v1/admins/student-parts-components/parts/{component_id}",
    responses(
        (status = 200, description = "Found parts for student component", body = GetPartsForComponentResponse),
        (status = 404, description = "Student component not found", body = JsonError),
        (status = 500, description = "Internal server error occurred", body = JsonError)
    ),
    security(("AdminAuth" = [])),
    tag = "Student parts-components management",
)]
/// Get all parts that use a specific student component.
///
/// Returns all student parts that use the specified component along with their quantities.
pub(super) async fn get_parts_for_component_handler(
    path: web::Path<i32>,
    data: Data<AppData>,
) -> Result<HttpResponse, JsonError> {
    let component_id = path.into_inner();

    // Verify the student component exists
    let component_exists = StudentsComponent::where_col(|sc| sc.students_component_id.equal(component_id))
        .run(&data.db)
        .await
        .map_err(|e| {
            error_with_log_id(
                format!("unable to check if student component exists: {}", e),
                "Failed to retrieve parts",
                StatusCode::INTERNAL_SERVER_ERROR,
                log::Level::Error,
            )
        })?;

    if component_exists.is_empty() {
        return Err("Student component not found".to_json_error(StatusCode::NOT_FOUND));
    }

    // Get all relationships for this component
    let relationships = StudentPartsComponent::where_col(|spc| {
        spc.students_component_id.equal(component_id)
    })
    .run(&data.db)
    .await
    .map_err(|e| {
        error_with_log_id(
            format!("unable to retrieve parts for component {}: {}", component_id, e),
            "Failed to retrieve parts",
            StatusCode::INTERNAL_SERVER_ERROR,
            log::Level::Error,
        )
    })?;

    let mut parts = Vec::new();

    for relationship in relationships {
        let relationship_data = DbState::into_inner(relationship);
        
        // Get component details
        let mut component_rows = StudentsComponent::where_col(|sc| {
            sc.students_component_id.equal(relationship_data.students_component_id)
        })
        .run(&data.db)
        .await
        .map_err(|e| {
            error_with_log_id(
                format!("unable to retrieve component details: {}", e),
                "Failed to retrieve parts",
                StatusCode::INTERNAL_SERVER_ERROR,
                log::Level::Error,
            )
        })?;

        let component = match component_rows.pop() {
            Some(c) => DbState::into_inner(c),
            None => continue, // Skip if component not found
        };

        // Get part details
        let mut part_rows = StudentPart::where_col(|sp| {
            sp.student_part_id.equal(relationship_data.student_part_id)
        })
        .run(&data.db)
        .await
        .map_err(|e| {
            error_with_log_id(
                format!("unable to retrieve part details: {}", e),
                "Failed to retrieve parts",
                StatusCode::INTERNAL_SERVER_ERROR,
                log::Level::Error,
            )
        })?;

        let part = match part_rows.pop() {
            Some(p) => DbState::into_inner(p),
            None => continue, // Skip if part not found
        };

        parts.push(StudentPartComponentResponse {
            id: relationship_data.id,
            student_part_id: relationship_data.student_part_id,
            students_component_id: relationship_data.students_component_id,
            quantity: relationship_data.quantity,
            component_name: component.name,
            part_name: part.name,
        });
    }

    Ok(HttpResponse::Ok().json(GetPartsForComponentResponse { parts }))
}
