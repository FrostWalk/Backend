use crate::app_data::AppData;
use crate::common::json_error::{error_with_log_id_and_payload, JsonError, ToJsonError};
use crate::models::student_deliverable::StudentDeliverable;
use actix_web::http::StatusCode;
use actix_web::web::{Data, Json};
use actix_web::HttpResponse;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use welds::state::DbState;

#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub(crate) struct CreateStudentDeliverableScheme {
    #[schema(example = "1")]
    pub project_id: i32,
    #[schema(example = "Motor")]
    pub name: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub(crate) struct CreateStudentDeliverableResponse {
    #[schema(example = "123")]
    pub student_deliverable_id: i32,
    #[schema(example = "1")]
    pub project_id: i32,
    #[schema(example = "Motor")]
    pub name: String,
}

#[utoipa::path(
    post,
    path = "/v1/admins/student-deliverables",
    request_body = CreateStudentDeliverableScheme,
    responses(
        (status = 200, description = "Student deliverable created successfully", body = CreateStudentDeliverableResponse),
        (status = 400, description = "Invalid data in request", body = JsonError),
        (status = 401, description = "Authentication required", body = JsonError),
        (status = 409, description = "Deliverable with this name already exists for the project", body = JsonError),
        (status = 500, description = "Internal server error occurred", body = JsonError)
    ),
    security(("AdminAuth" = [])),
    tag = "Student deliverables management",
)]
/// Creates a new student deliverable.
///
/// This endpoint allows authenticated admins to create a new student deliverable for a specific project.
#[actix_web_grants::protect(any("ROLE_ADMIN_ROOT", "ROLE_ADMIN_PROFESSOR"))]
pub(super) async fn create_student_deliverable_handler(
    body: Json<CreateStudentDeliverableScheme>, data: Data<AppData>,
) -> Result<HttpResponse, JsonError> {
    // Check if deliverable with this name already exists for the project
    let existing = StudentDeliverable::where_col(|sp| sp.project_id.equal(body.project_id))
        .where_col(|sp| sp.name.equal(&body.name))
        .run(&data.db)
        .await
        .map_err(|e| {
            error_with_log_id_and_payload(
                format!("unable to check existing student deliverable: {}", e),
                "Failed to create deliverable",
                StatusCode::INTERNAL_SERVER_ERROR,
                log::Level::Error,
                &body,
            )
        })?;

    if !existing.is_empty() {
        return Err("Deliverable with this name already exists for the project"
            .to_json_error(StatusCode::CONFLICT));
    }

    let mut state = DbState::new_uncreated(StudentDeliverable {
        student_deliverable_id: 0,
        project_id: body.project_id,
        name: body.name.clone(),
    });

    if let Err(e) = state.save(&data.db).await {
        return Err(error_with_log_id_and_payload(
            format!("unable to create student deliverable: {}", e),
            "Failed to create deliverable",
            StatusCode::INTERNAL_SERVER_ERROR,
            log::Level::Error,
            &body,
        ));
    }

    Ok(HttpResponse::Ok().json(CreateStudentDeliverableResponse {
        student_deliverable_id: state.student_deliverable_id,
        project_id: body.project_id,
        name: body.name.clone(),
    }))
}
