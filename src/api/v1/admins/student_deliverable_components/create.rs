use crate::app_data::AppData;
use crate::common::json_error::{error_with_log_id_and_payload, JsonError, ToJsonError};
use crate::models::student_deliverable_component::StudentDeliverableComponent;
use actix_web::http::StatusCode;
use actix_web::web::{Data, Json};
use actix_web::HttpResponse;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use welds::state::DbState;

#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub(crate) struct CreateStudentComponentScheme {
    #[schema(example = "1")]
    pub project_id: i32,
    #[schema(example = "Robot")]
    pub name: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub(crate) struct CreateStudentComponentResponse {
    #[schema(example = "123")]
    pub student_deliverable_component_id: i32,
    #[schema(example = "1")]
    pub project_id: i32,
    #[schema(example = "Robot")]
    pub name: String,
}

#[utoipa::path(
    post,
    path = "/v1/admins/student-deliverable-components",
    request_body = CreateStudentComponentScheme,
    responses(
        (status = 200, description = "Student deliverable component created successfully", body = CreateStudentComponentResponse),
        (status = 400, description = "Invalid data in request", body = JsonError),
        (status = 401, description = "Authentication required", body = JsonError),
        (status = 409, description = "Deliverable component with this name already exists for the project", body = JsonError),
        (status = 500, description = "Internal server error occurred", body = JsonError)
    ),
    security(("AdminAuth" = [])),
    tag = "Student deliverable components management",
)]
/// Creates a new student component.
///
/// This endpoint allows authenticated admins to create a new student component for a specific project.
pub(super) async fn create_student_component_handler(
    body: Json<CreateStudentComponentScheme>, 
    data: Data<AppData>,
) -> Result<HttpResponse, JsonError> {
    // Check if component with this name already exists for the project
    let existing = StudentDeliverableComponent::where_col(|sc| sc.project_id.equal(body.project_id))
        .where_col(|sc| sc.name.equal(&body.name))
        .run(&data.db)
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

    if !existing.is_empty() {
        return Err("Component with this name already exists for the project"
            .to_json_error(StatusCode::CONFLICT));
    }

    let mut state = DbState::new_uncreated(StudentDeliverableComponent {
        student_deliverable_component_id: 0,
        project_id: body.project_id,
        name: body.name.clone(),
    });

    if let Err(e) = state.save(&data.db).await {
        return Err(error_with_log_id_and_payload(
            format!("unable to create student component: {}", e),
            "Failed to create component",
            StatusCode::INTERNAL_SERVER_ERROR,
            log::Level::Error,
            &body,
        ));
    }

    Ok(HttpResponse::Ok().json(CreateStudentComponentResponse {
        student_deliverable_component_id: state.student_deliverable_component_id,
        project_id: body.project_id,
        name: body.name.clone(),
    }))
}
