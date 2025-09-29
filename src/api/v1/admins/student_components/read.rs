use crate::app_data::AppData;
use crate::common::json_error::{error_with_log_id, JsonError, ToJsonError};
use crate::models::students_component::StudentsComponent;
use actix_web::http::StatusCode;
use actix_web::web::Data;
use actix_web::{web, HttpResponse};
use serde::Serialize;
use utoipa::ToSchema;
use welds::state::DbState;

#[derive(Debug, Serialize, ToSchema)]
pub(crate) struct StudentComponentResponse {
    #[schema(example = "123")]
    pub students_component_id: i32,
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

#[utoipa::path(
    get,
    path = "/v1/admins/student-components",
    responses(
        (status = 200, description = "Found all student components", body = GetAllStudentComponentsResponse),
        (status = 500, description = "Internal server error occurred", body = JsonError)
    ),
    security(("AdminAuth" = [])),
    tag = "Student components management",
)]
/// Get all student components.
///
/// Returns all student components across all projects.
pub(super) async fn get_all_student_components_handler(
    data: Data<AppData>,
) -> Result<HttpResponse, JsonError> {
    let components = StudentsComponent::all().run(&data.db).await.map_err(|e| {
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
            students_component_id: component.students_component_id,
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
    path = "/v1/admins/student-components/project/{project_id}",
    responses(
        (status = 200, description = "Found student components for project", body = GetStudentComponentsForProjectResponse),
        (status = 404, description = "Project not found", body = JsonError),
        (status = 500, description = "Internal server error occurred", body = JsonError)
    ),
    security(("AdminAuth" = [])),
    tag = "Student components management",
)]
/// Get all student components for a specific project.
///
/// Returns all student components associated with the specified project.
pub(super) async fn get_student_components_for_project_handler(
    path: web::Path<i32>, data: Data<AppData>,
) -> Result<HttpResponse, JsonError> {
    let project_id = path.into_inner();

    // Get all components for this project
    let components = StudentsComponent::where_col(|sc| sc.project_id.equal(project_id))
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
            students_component_id: component_data.students_component_id,
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
    path = "/v1/admins/student-components/{id}",
    responses(
        (status = 200, description = "Found student component", body = StudentComponentResponse),
        (status = 404, description = "Student component not found", body = JsonError),
        (status = 500, description = "Internal server error occurred", body = JsonError)
    ),
    security(("AdminAuth" = [])),
    tag = "Student components management",
)]
/// Get a specific student component by ID.
///
/// Returns the details of the specified student component.
pub(super) async fn get_student_component_handler(
    path: web::Path<i32>, data: Data<AppData>,
) -> Result<HttpResponse, JsonError> {
    let component_id = path.into_inner();

    // Get the component by ID
    let mut components =
        StudentsComponent::where_col(|sc| sc.students_component_id.equal(component_id))
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
        students_component_id: component.students_component_id,
        project_id: component.project_id,
        name: component.name,
    }))
}
