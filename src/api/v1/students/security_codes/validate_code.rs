use crate::app_data::AppData;
use crate::common::json_error::{error_with_log_id, JsonError};
use crate::database::repositories::{projects_repository, security_codes};
use crate::jwt::get_user::LoggedUser;
use actix_web::http::StatusCode;
use actix_web::web::{Data, Json};
use actix_web::{HttpMessage, HttpRequest, HttpResponse};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use welds::state::DbState;

#[derive(Debug, Deserialize, ToSchema)]
pub(crate) struct ValidateCodeRequest {
    pub security_code: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub(crate) struct ValidateCodeResponse {
    pub is_valid: bool,
    pub project: Option<ProjectInfo>,
}

#[derive(Debug, Serialize, ToSchema)]
pub(crate) struct ProjectInfo {
    pub project_id: i32,
    pub name: String,
    pub year: i32,
}

#[utoipa::path(
    post,
    path = "/v1/students/security-codes/validate",
    request_body = ValidateCodeRequest,
    responses(
        (status = 200, description = "Security code validation result", body = ValidateCodeResponse),
        (status = 401, description = "Authentication required", body = JsonError),
        (status = 500, description = "Internal server error", body = JsonError)
    ),
    security(("UserAuth" = [])),
    tag = "Security codes management",
)]
/// Validate a security code and return project information
///
/// This endpoint allows students to validate a security code and get information about
/// the project associated with it. All security codes are for GroupLeader role.
pub(super) async fn validate_code(
    req: HttpRequest, data: Data<AppData>, body: Json<ValidateCodeRequest>,
) -> Result<HttpResponse, JsonError> {
    let _user = match req.extensions().get_student() {
        Ok(user) => user,
        Err(_) => {
            return Err(error_with_log_id(
                "entered a protected route without a user loaded in the request",
                "Authentication error",
                StatusCode::INTERNAL_SERVER_ERROR,
                log::Level::Error,
            ));
        }
    };

    // Find the security code
    let security_code_state = security_codes::get_by_code(&data.db, &body.security_code)
        .await
        .map_err(|e| {
            error_with_log_id(
                format!("unable to validate security code: {}", e),
                "Database error",
                StatusCode::INTERNAL_SERVER_ERROR,
                log::Level::Error,
            )
        })?;

    let security_code = match security_code_state {
        Some(state) => DbState::into_inner(state),
        None => {
            return Ok(HttpResponse::Ok().json(ValidateCodeResponse {
                is_valid: false,
                project: None,
            }));
        }
    };

    // Check if the security code has expired
    if security_code.expiration <= Utc::now() {
        return Ok(HttpResponse::Ok().json(ValidateCodeResponse {
            is_valid: false,
            project: None,
        }));
    }

    // Get the project information
    let project_state = projects_repository::get_by_id(&data.db, security_code.project_id)
        .await
        .map_err(|e| {
            error_with_log_id(
                format!("unable to fetch project information: {}", e),
                "Database error",
                StatusCode::INTERNAL_SERVER_ERROR,
                log::Level::Error,
            )
        })?;

    let project = match project_state {
        Some(state) => {
            let project_data = DbState::into_inner(state);
            Some(ProjectInfo {
                project_id: project_data.project_id,
                name: project_data.name,
                year: project_data.year,
            })
        }
        None => None,
    };

    // All security codes are for GroupLeader role
    Ok(HttpResponse::Ok().json(ValidateCodeResponse {
        is_valid: true,
        project,
    }))
}
