use crate::app_data::AppData;
use crate::common::json_error::{error_with_log_id_and_payload, JsonError, ToJsonError};
use crate::database::repositories::student_deliverable_components_repository;
use actix_web::http::StatusCode;
use actix_web::web::Path;
use actix_web::web::{Data, Json};
use actix_web::HttpResponse;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub(crate) struct UpdateStudentComponentScheme {
    #[schema(example = "Updated Resistor")]
    pub name: String,
}

#[utoipa::path(
    patch,
    path = "/v1/admins/student-deliverable-components/{id}",
    request_body = UpdateStudentComponentScheme,
    responses(
        (status = 200, description = "Component updated successfully"),
        (status = 400, description = "Invalid data in request", body = JsonError),
        (status = 401, description = "Authentication required", body = JsonError),
        (status = 404, description = "Student component not found", body = JsonError),
        (status = 409, description = "Component with this name already exists for the project", body = JsonError),
        (status = 500, description = "Internal server error occurred", body = JsonError)
    ),
    security(("AdminAuth" = [])),
    tag = "Student deliverable components management",
)]
/// Updates a student component.
///
/// This endpoint allows authenticated admins to modify the name of a student component by ID.
#[actix_web_grants::protect(any("ROLE_ADMIN_ROOT", "ROLE_ADMIN_PROFESSOR"))]
pub(super) async fn update_student_component_handler(
    path: Path<i32>, body: Json<UpdateStudentComponentScheme>, data: Data<AppData>,
) -> Result<HttpResponse, JsonError> {
    let id = path.into_inner();

    // Find the existing component by ID
    let mut component_state = student_deliverable_components_repository::get_by_id(&data.db, id)
        .await
        .map_err(|e| {
            error_with_log_id_and_payload(
                format!("unable to load student component: {}", e),
                "Failed to update component",
                StatusCode::INTERNAL_SERVER_ERROR,
                log::Level::Error,
                &body,
            )
        })?
        .ok_or_else(|| "Student component not found".to_json_error(StatusCode::NOT_FOUND))?;

    // Check if another component with this name already exists for the same project
    let exists = student_deliverable_components_repository::check_name_exists_excluding(
        &data.db,
        component_state.project_id,
        &body.name,
        id,
    )
    .await
    .map_err(|e| {
        error_with_log_id_and_payload(
            format!("unable to check existing component: {}", e),
            "Failed to update component",
            StatusCode::INTERNAL_SERVER_ERROR,
            log::Level::Error,
            &body,
        )
    })?;

    if exists {
        return Err("Component with this name already exists for the project"
            .to_json_error(StatusCode::CONFLICT));
    }

    // Update the name
    component_state.name = body.name.clone();

    student_deliverable_components_repository::update(&data.db, component_state)
        .await
        .map_err(|e| {
            error_with_log_id_and_payload(
                format!("unable to update student component: {}", e),
                "Failed to update component",
                StatusCode::INTERNAL_SERVER_ERROR,
                log::Level::Error,
                &body,
            )
        })?;

    Ok(HttpResponse::Ok().finish())
}
