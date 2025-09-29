use crate::app_data::AppData;
use crate::common::json_error::{error_with_log_id, JsonError};
use crate::jwt::get_user::LoggedUser;
use crate::models::project::Project;
use crate::models::security_code::SecurityCode;
use crate::models::student_role::{AvailableStudentRole, StudentRole};
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
    pub role: Option<String>,
    pub project: Option<ProjectInfo>,
    pub message: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub(crate) struct ProjectInfo {
    pub project_id: i32,
    pub name: String,
    pub year: i32,
}

#[utoipa::path(
    post,
    path = "/v1/students/groups/validate-code",
    request_body = ValidateCodeRequest,
    responses(
        (status = 200, description = "Security code validation result", body = ValidateCodeResponse),
        (status = 401, description = "Authentication required", body = JsonError),
        (status = 500, description = "Internal server error", body = JsonError)
    ),
    security(("UserAuth" = [])),
    tag = "Groups management",
)]
/// Validate a security code and return role information
///
/// This endpoint allows students to validate a security code and get information about
/// the role and project associated with it.
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
    let security_code = match SecurityCode::where_col(|sc| sc.code.equal(&body.security_code))
        .run(&data.db)
        .await
    {
        Ok(mut rows) => match rows.pop() {
            Some(state) => DbState::into_inner(state),
            None => {
                return Ok(HttpResponse::Ok().json(ValidateCodeResponse {
                    is_valid: false,
                    role: None,
                    project: None,
                    message: "Invalid security code".to_string(),
                }));
            }
        },
        Err(e) => {
            return Err(error_with_log_id(
                format!("unable to validate security code: {}", e),
                "Database error",
                StatusCode::INTERNAL_SERVER_ERROR,
                log::Level::Error,
            ));
        }
    };

    // Check if the security code has expired
    if security_code.expiration <= Utc::now() {
        return Ok(HttpResponse::Ok().json(ValidateCodeResponse {
            is_valid: false,
            role: None,
            project: None,
            message: "Security code has expired".to_string(),
        }));
    }

    // Get the role information
    let role =
        match StudentRole::where_col(|sr| sr.student_role_id.equal(security_code.student_role_id))
            .run(&data.db)
            .await
        {
            Ok(mut rows) => match rows.pop() {
                Some(state) => Some(DbState::into_inner(state).name),
                None => None,
            },
            Err(e) => {
                return Err(error_with_log_id(
                    format!("unable to fetch role information: {}", e),
                    "Database error",
                    StatusCode::INTERNAL_SERVER_ERROR,
                    log::Level::Error,
                ));
            }
        };

    // Get the project information
    let project = match Project::where_col(|p| p.project_id.equal(security_code.project_id))
        .run(&data.db)
        .await
    {
        Ok(mut rows) => match rows.pop() {
            Some(state) => {
                let project_data = DbState::into_inner(state);
                Some(ProjectInfo {
                    project_id: project_data.project_id,
                    name: project_data.name,
                    year: project_data.year,
                })
            }
            None => None,
        },
        Err(e) => {
            return Err(error_with_log_id(
                format!("unable to fetch project information: {}", e),
                "Database error",
                StatusCode::INTERNAL_SERVER_ERROR,
                log::Level::Error,
            ));
        }
    };

    // Check if the role is GroupLeader
    let is_group_leader = security_code.student_role_id == AvailableStudentRole::GroupLeader as i32;

    Ok(HttpResponse::Ok().json(ValidateCodeResponse {
        is_valid: is_group_leader,
        role,
        project,
        message: if is_group_leader {
            "Valid GroupLeader security code".to_string()
        } else {
            "Security code is valid but not for GroupLeader role".to_string()
        },
    }))
}
