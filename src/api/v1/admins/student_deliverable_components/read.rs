use crate::app_data::AppData;
use crate::common::json_error::{error_with_log_id, JsonError, ToJsonError};
use crate::models::student_deliverable_component::StudentDeliverableComponent;
use crate::models::student_deliverables_component::StudentDeliverablesComponent;
use actix_web::http::StatusCode;
use actix_web::web::Data;
use actix_web::web::Path;
use actix_web::HttpResponse;
use serde::Serialize;
use utoipa::ToSchema;
use welds::state::DbState;

#[derive(Debug, Serialize, ToSchema)]
pub(crate) struct StudentComponentResponse {
    #[schema(example = "123")]
    pub student_deliverable_component_id: i32,
    #[schema(example = "1")]
    pub project_id: i32,
    #[schema(example = "Resistor")]
    pub name: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub(crate) struct GetAllStudentComponentsResponse {
    pub components: Vec<StudentComponentResponse>,
}

#[derive(Debug, Serialize, ToSchema)]
pub(crate) struct GetStudentComponentsForProjectResponse {
    pub components: Vec<StudentComponentResponse>,
}

#[derive(Debug, Serialize, ToSchema)]
pub(crate) struct StudentComponentDeliverableResponse {
    #[schema(example = "123")]
    pub id: i32,
    #[schema(example = "1")]
    pub student_deliverable_id: i32,
    #[schema(example = "2")]
    pub student_deliverable_component_id: i32,
    #[schema(example = "5")]
    pub quantity: i32,
    #[schema(example = "Motor")]
    pub deliverable_name: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub(crate) struct GetDeliverablesForStudentComponentResponse {
    pub deliverables: Vec<StudentComponentDeliverableResponse>,
}

#[utoipa::path(
    get,
    path = "/v1/admins/student-deliverable-components",
    responses(
        (status = 200, description = "Found all student components", body = GetAllStudentComponentsResponse),
        (status = 500, description = "Internal server error occurred", body = JsonError)
    ),
    security(("AdminAuth" = [])),
    tag = "Student deliverable components management",
)]
/// Get all student components.
///
/// Returns all student components across all projects.
pub(super) async fn get_all_student_components_handler(
    data: Data<AppData>,
) -> Result<HttpResponse, JsonError> {
    let components = StudentDeliverableComponent::all()
        .run(&data.db)
        .await
        .map_err(|e| {
            error_with_log_id(
                format!("unable to retrieve all student components: {}", e),
                "Failed to retrieve components",
                StatusCode::INTERNAL_SERVER_ERROR,
                log::Level::Error,
            )
        })?;

    let response_components: Vec<StudentComponentResponse> = components
        .into_iter()
        .map(DbState::into_inner)
        .map(|component| StudentComponentResponse {
            student_deliverable_component_id: component.student_deliverable_component_id,
            project_id: component.project_id,
            name: component.name,
        })
        .collect();

    Ok(HttpResponse::Ok().json(GetAllStudentComponentsResponse {
        components: response_components,
    }))
}

#[utoipa::path(
    get,
    path = "/v1/admins/student-deliverable-components/project/{project_id}",
    responses(
        (status = 200, description = "Found student components for project", body = GetStudentComponentsForProjectResponse),
        (status = 404, description = "Project not found", body = JsonError),
        (status = 500, description = "Internal server error occurred", body = JsonError)
    ),
    security(("AdminAuth" = [])),
    tag = "Student deliverable components management",
)]
/// Get all student components for a specific project.
///
/// Returns all student components associated with the specified project.
pub(super) async fn get_student_components_for_project_handler(
    path: Path<i32>, data: Data<AppData>,
) -> Result<HttpResponse, JsonError> {
    let project_id = path.into_inner();

    // Get all components for this project
    let components = StudentDeliverableComponent::where_col(|sc| sc.project_id.equal(project_id))
        .run(&data.db)
        .await
        .map_err(|e| {
            error_with_log_id(
                format!(
                    "unable to retrieve components for project {}: {}",
                    project_id, e
                ),
                "Failed to retrieve components",
                StatusCode::INTERNAL_SERVER_ERROR,
                log::Level::Error,
            )
        })?;

    let mut response_components = Vec::new();

    for component in components {
        let component_data = DbState::into_inner(component);
        response_components.push(StudentComponentResponse {
            student_deliverable_component_id: component_data.student_deliverable_component_id,
            project_id: component_data.project_id,
            name: component_data.name,
        });
    }

    Ok(
        HttpResponse::Ok().json(GetStudentComponentsForProjectResponse {
            components: response_components,
        }),
    )
}

#[utoipa::path(
    get,
    path = "/v1/admins/student-deliverable-components/{id}",
    responses(
        (status = 200, description = "Found student component", body = StudentComponentResponse),
        (status = 404, description = "Student component not found", body = JsonError),
        (status = 500, description = "Internal server error occurred", body = JsonError)
    ),
    security(("AdminAuth" = [])),
    tag = "Student deliverable components management",
)]
/// Get a specific student component by ID.
///
/// Returns the details of the specified student component.
pub(super) async fn get_student_component_handler(
    path: Path<i32>, data: Data<AppData>,
) -> Result<HttpResponse, JsonError> {
    let component_id = path.into_inner();

    // Get the component by ID
    let mut components = StudentDeliverableComponent::where_col(|sc| {
        sc.student_deliverable_component_id.equal(component_id)
    })
    .run(&data.db)
    .await
    .map_err(|e| {
        error_with_log_id(
            format!("unable to retrieve component {}: {}", component_id, e),
            "Failed to retrieve component",
            StatusCode::INTERNAL_SERVER_ERROR,
            log::Level::Error,
        )
    })?;

    let component = match components.pop() {
        Some(c) => DbState::into_inner(c),
        None => return Err("Student component not found".to_json_error(StatusCode::NOT_FOUND)),
    };

    Ok(HttpResponse::Ok().json(StudentComponentResponse {
        student_deliverable_component_id: component.student_deliverable_component_id,
        project_id: component.project_id,
        name: component.name,
    }))
}

#[utoipa::path(
    get,
    path = "/v1/admins/student-deliverable-components/{id}/deliverables",
    responses(
        (status = 200, description = "Found deliverables for student component", body = GetDeliverablesForStudentComponentResponse),
        (status = 404, description = "Student component not found", body = JsonError),
        (status = 500, description = "Internal server error occurred", body = JsonError)
    ),
    security(("AdminAuth" = [])),
    tag = "Student deliverable components management",
)]
/// Get all deliverables that use a specific student component.
///
/// Returns all student deliverables that use the specified component along with their quantities.
pub(super) async fn get_deliverables_for_student_component_handler(
    path: Path<i32>, data: Data<AppData>,
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

    // Efficiently fetch relationships with related entities using map_query
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

    let deliverables_data = StudentDeliverablesComponent::where_col(|spc| {
        spc.student_deliverable_component_id.equal(component_id)
    })
    .map_query(|spc| spc.student_deliverable)
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

    for (relationship_state, deliverable_state) in relationships.into_iter().zip(deliverables_data)
    {
        let relationship_data = DbState::into_inner(relationship_state);
        let deliverable = DbState::into_inner(deliverable_state);

        deliverables.push(StudentComponentDeliverableResponse {
            id: relationship_data.id,
            student_deliverable_id: relationship_data.student_deliverable_id,
            student_deliverable_component_id: relationship_data.student_deliverable_component_id,
            quantity: relationship_data.quantity,
            deliverable_name: deliverable.name,
        });
    }

    Ok(HttpResponse::Ok().json(GetDeliverablesForStudentComponentResponse { deliverables }))
}
