use crate::app_data::AppData;
use crate::common::json_error::{error_with_log_id_and_payload, JsonError, ToJsonError};
use crate::models::student_part::StudentPart;
use actix_web::http::StatusCode;
use actix_web::web::{Data, Json};
use actix_web::HttpResponse;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use welds::state::DbState;

#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub(crate) struct CreateStudentPartScheme {
    #[schema(example = "1")]
    pub project_id: i32,
    #[schema(example = "Motor")]
    pub name: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub(crate) struct CreateStudentPartResponse {
    #[schema(example = "123")]
    pub student_part_id: i32,
    #[schema(example = "1")]
    pub project_id: i32,
    #[schema(example = "Motor")]
    pub name: String,
}

#[utoipa::path(
    post,
    path = "/v1/admins/student-parts",
    request_body = CreateStudentPartScheme,
    responses(
        (status = 200, description = "Student part created successfully", body = CreateStudentPartResponse),
        (status = 400, description = "Invalid data in request", body = JsonError),
        (status = 401, description = "Authentication required", body = JsonError),
        (status = 409, description = "Part with this name already exists for the project", body = JsonError),
        (status = 500, description = "Internal server error occurred", body = JsonError)
    ),
    security(("AdminAuth" = [])),
    tag = "Student parts management",
)]
/// Creates a new student part.
///
/// This endpoint allows authenticated admins to create a new student part for a specific project.
pub(super) async fn create_student_part_handler(
    payload: Json<CreateStudentPartScheme>, data: Data<AppData>,
) -> Result<HttpResponse, JsonError> {
    let scheme = payload.into_inner();
    let original_payload = Json(CreateStudentPartScheme {
        project_id: scheme.project_id,
        name: scheme.name.clone(),
    });

    // Check if part with this name already exists for the project
    let existing = StudentPart::where_col(|sp| sp.project_id.equal(scheme.project_id))
        .where_col(|sp| sp.name.equal(&scheme.name))
        .run(&data.db)
        .await
        .map_err(|e| {
            error_with_log_id_and_payload(
                format!("unable to check existing student part: {}", e),
                "Failed to create part",
                StatusCode::INTERNAL_SERVER_ERROR,
                log::Level::Error,
                &original_payload,
            )
        })?;

    if !existing.is_empty() {
        return Err("Part with this name already exists for the project"
            .to_json_error(StatusCode::CONFLICT));
    }

    let mut state = DbState::new_uncreated(StudentPart {
        student_part_id: 0,
        project_id: scheme.project_id,
        name: scheme.name.clone(),
    });

    if let Err(e) = state.save(&data.db).await {
        return Err(error_with_log_id_and_payload(
            format!("unable to create student part: {}", e),
            "Failed to create part",
            StatusCode::INTERNAL_SERVER_ERROR,
            log::Level::Error,
            &original_payload,
        ));
    }

    Ok(HttpResponse::Ok().json(CreateStudentPartResponse {
        student_part_id: state.student_part_id,
        project_id: scheme.project_id,
        name: scheme.name,
    }))
}
