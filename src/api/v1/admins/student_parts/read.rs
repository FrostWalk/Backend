use crate::app_data::AppData;
use crate::common::json_error::{error_with_log_id, JsonError, ToJsonError};
use crate::models::student_part::StudentPart;
use crate::models::student_parts_component::StudentPartsComponent;
use crate::models::students_component::StudentsComponent;
use actix_web::http::StatusCode;
use actix_web::web::Data;
use actix_web::{web, HttpResponse};
use serde::Serialize;
use utoipa::ToSchema;
use welds::state::DbState;

#[derive(Debug, Serialize, ToSchema)]
pub(crate) struct StudentPartResponse {
    #[schema(example = "123")]
    pub student_part_id: i32,
    #[schema(example = "1")]
    pub project_id: i32,
    #[schema(example = "Motor")]
    pub name: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub(crate) struct GetAllStudentPartsResponse {
    pub parts: Vec<StudentPartResponse>,
}

#[derive(Debug, Serialize, ToSchema)]
pub(crate) struct GetStudentPartsForProjectResponse {
    pub parts: Vec<StudentPartResponse>,
}

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
}

#[derive(Debug, Serialize, ToSchema)]
pub(crate) struct GetComponentsForStudentPartResponse {
    pub components: Vec<StudentPartComponentResponse>,
}

#[utoipa::path(
    get,
    path = "/v1/admins/student-parts",
    responses(
        (status = 200, description = "Found all student parts", body = GetAllStudentPartsResponse),
        (status = 500, description = "Internal server error occurred", body = JsonError)
    ),
    security(("AdminAuth" = [])),
    tag = "Student parts management",
)]
/// Get all student parts.
///
/// Returns all student parts across all projects.
pub(super) async fn get_all_student_parts_handler(
    data: Data<AppData>,
) -> Result<HttpResponse, JsonError> {
    let parts = StudentPart::all().run(&data.db).await.map_err(|e| {
        error_with_log_id(
            format!("unable to retrieve all student parts: {}", e),
            "Failed to retrieve parts",
            StatusCode::INTERNAL_SERVER_ERROR,
            log::Level::Error,
        )
    })?;

    let response_parts: Vec<StudentPartResponse> = parts
        .into_iter()
        .map(DbState::into_inner)
        .map(|part| StudentPartResponse {
            student_part_id: part.student_part_id,
            project_id: part.project_id,
            name: part.name,
        })
        .collect();

    Ok(HttpResponse::Ok().json(GetAllStudentPartsResponse {
        parts: response_parts,
    }))
}

#[utoipa::path(
    get,
    path = "/v1/admins/student-parts/project/{project_id}",
    responses(
        (status = 200, description = "Found student parts for project", body = GetStudentPartsForProjectResponse),
        (status = 404, description = "Project not found", body = JsonError),
        (status = 500, description = "Internal server error occurred", body = JsonError)
    ),
    security(("AdminAuth" = [])),
    tag = "Student parts management",
)]
/// Get all student parts for a specific project.
///
/// Returns all student parts associated with the specified project.
pub(super) async fn get_student_parts_for_project_handler(
    path: web::Path<i32>, data: Data<AppData>,
) -> Result<HttpResponse, JsonError> {
    let project_id = path.into_inner();

    // Get all parts for this project
    let parts = StudentPart::where_col(|sp| sp.project_id.equal(project_id))
        .run(&data.db)
        .await
        .map_err(|e| {
            error_with_log_id(
                format!(
                    "unable to retrieve parts for project {}: {}",
                    project_id, e
                ),
                "Failed to retrieve parts",
                StatusCode::INTERNAL_SERVER_ERROR,
                log::Level::Error,
            )
        })?;

    let mut response_parts = Vec::new();

    for part in parts {
        let part_data = DbState::into_inner(part);
        response_parts.push(StudentPartResponse {
            student_part_id: part_data.student_part_id,
            project_id: part_data.project_id,
            name: part_data.name,
        });
    }

    Ok(
        HttpResponse::Ok().json(GetStudentPartsForProjectResponse {
            parts: response_parts,
        }),
    )
}

#[utoipa::path(
    get,
    path = "/v1/admins/student-parts/{id}",
    responses(
        (status = 200, description = "Found student part", body = StudentPartResponse),
        (status = 404, description = "Student part not found", body = JsonError),
        (status = 500, description = "Internal server error occurred", body = JsonError)
    ),
    security(("AdminAuth" = [])),
    tag = "Student parts management",
)]
/// Get a specific student part by ID.
///
/// Returns the details of the specified student part.
pub(super) async fn get_student_part_handler(
    path: web::Path<i32>, data: Data<AppData>,
) -> Result<HttpResponse, JsonError> {
    let part_id = path.into_inner();

    // Get the part by ID
    let mut parts =
        StudentPart::where_col(|sp| sp.student_part_id.equal(part_id))
            .run(&data.db)
            .await
            .map_err(|e| {
                error_with_log_id(
                    format!("unable to retrieve part {}: {}", part_id, e),
                    "Failed to retrieve part",
                    StatusCode::INTERNAL_SERVER_ERROR,
                    log::Level::Error,
                )
            })?;

    let part = match parts.pop() {
        Some(p) => DbState::into_inner(p),
        None => return Err("Student part not found".to_json_error(StatusCode::NOT_FOUND)),
    };

    Ok(HttpResponse::Ok().json(StudentPartResponse {
        student_part_id: part.student_part_id,
        project_id: part.project_id,
        name: part.name,
    }))
}

#[utoipa::path(
    get,
    path = "/v1/admins/student-parts/{id}/components",
    responses(
        (status = 200, description = "Found components for student part", body = GetComponentsForStudentPartResponse),
        (status = 404, description = "Student part not found", body = JsonError),
        (status = 500, description = "Internal server error occurred", body = JsonError)
    ),
    security(("AdminAuth" = [])),
    tag = "Student parts management",
)]
/// Get all components for a specific student part.
///
/// Returns all components associated with the specified student part along with their quantities.
pub(super) async fn get_components_for_student_part_handler(
    path: web::Path<i32>, data: Data<AppData>,
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
    let relationships = StudentPartsComponent::where_col(|spc| spc.student_part_id.equal(part_id))
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
            sc.students_component_id
                .equal(relationship_data.students_component_id)
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

        components.push(StudentPartComponentResponse {
            id: relationship_data.id,
            student_part_id: relationship_data.student_part_id,
            students_component_id: relationship_data.students_component_id,
            quantity: relationship_data.quantity,
            component_name: component.name,
        });
    }

    Ok(HttpResponse::Ok().json(GetComponentsForStudentPartResponse { components }))
}
