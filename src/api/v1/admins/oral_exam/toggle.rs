use crate::app_data::AppData;
use crate::common::json_error::{error_with_log_id, JsonError};
use crate::database::repositories::projects_repository;
use crate::jwt::get_user::LoggedUser;
use actix_web::http::StatusCode;
use actix_web::web::{Data, Json, Path};
use actix_web::{HttpMessage, HttpRequest, HttpResponse};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Deserialize, ToSchema)]
pub(crate) struct ToggleOralExamRequest {
    pub enabled: bool,
}

#[derive(Debug, Serialize, ToSchema)]
pub(crate) struct ToggleOralExamResponse {
    pub project_id: i32,
    pub oral_exam_enabled: bool,
}

#[utoipa::path(
    patch,
    path = "/v1/admins/oral-exam/projects/{project_id}",
    request_body = ToggleOralExamRequest,
    responses(
        (status = 200, description = "Oral exam mode updated", body = ToggleOralExamResponse),
        (status = 401, description = "Authentication required", body = JsonError),
        (status = 404, description = "Project not found", body = JsonError),
        (status = 500, description = "Internal server error", body = JsonError)
    ),
    security(("AdminAuth" = [])),
    tag = "Admin Oral Exam",
)]
#[actix_web_grants::protect(any("ROLE_ADMIN_ROOT", "ROLE_ADMIN_PROFESSOR"))]
pub(super) async fn toggle_oral_exam(
    req: HttpRequest, path: Path<i32>, body: Json<ToggleOralExamRequest>, data: Data<AppData>,
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

    let project_id = path.into_inner();

    let mut project_state = projects_repository::get_by_id(&data.db, project_id)
        .await
        .map_err(|e| {
            error_with_log_id(
                format!("unable to fetch project {}: {}", project_id, e),
                "Database error",
                StatusCode::INTERNAL_SERVER_ERROR,
                log::Level::Error,
            )
        })?
        .ok_or_else(|| {
            error_with_log_id(
                format!("project {} not found", project_id),
                "Project not found",
                StatusCode::NOT_FOUND,
                log::Level::Warn,
            )
        })?;

    project_state.as_mut().oral_exam_enabled = body.enabled;
    project_state.save(&data.db).await.map_err(|e| {
        error_with_log_id(
            format!("unable to save project {}: {}", project_id, e),
            "Database error",
            StatusCode::INTERNAL_SERVER_ERROR,
            log::Level::Error,
        )
    })?;

    Ok(HttpResponse::Ok().json(ToggleOralExamResponse {
        project_id,
        oral_exam_enabled: body.enabled,
    }))
}
