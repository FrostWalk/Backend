use crate::app_data::AppData;
use crate::common::json_error::{error_with_log_id, JsonError, ToJsonError};
use crate::database::repositories::coordinator_projects_repository;
use crate::database::repositories::security_codes::{delete as delete_security_code, get_by_id};
use crate::jwt::get_user::LoggedUser;
use crate::models::admin_role::AvailableAdminRole;
use actix_web::http::StatusCode;
use actix_web::web::{Data, Path};
use actix_web::{HttpMessage, HttpRequest, HttpResponse};
use log::error;
use serde::Serialize;
use utoipa::ToSchema;
use welds::state::DbState;

#[derive(Debug, Serialize, ToSchema)]
pub(crate) struct DeleteCodeResponse {
    #[schema(example = "Security code deleted successfully")]
    pub message: String,
}

#[utoipa::path(
    delete,
    path = "/v1/admins/security-codes/{security_code_id}",
    responses(
        (status = 200, description = "Code deleted successfully", body = DeleteCodeResponse),
        (status = 403, description = "Access denied", body = JsonError),
        (status = 404, description = "Security code not found", body = JsonError),
        (status = 500, description = "Internal server error", body = JsonError)
    ),
    security(("AdminAuth" = [])),
    tag = "Security codes management",
)]
/// Delete a security code
///
/// Coordinators can only delete codes for projects they are assigned to. Professors/Root can delete codes for any project.
pub(in crate::api::v1) async fn delete_code_handler(
    req: HttpRequest, 
    path: Path<i32>, 
    data: Data<AppData>,
) -> Result<HttpResponse, JsonError> {
    let user = match req.extensions().get_admin() {
        Ok(user) => user,
        Err(e) => {
            error!("entered a protected route without a user loaded in the request");
            return Err(e.to_json_error(StatusCode::INTERNAL_SERVER_ERROR));
        }
    };

    let security_code_id = path.into_inner();

    // Get the existing security code to check project access
    let existing_code = match get_by_id(&data.db, security_code_id).await {
        Ok(Some(code)) => code,
        Ok(None) => {
            return Err("Security code not found".to_json_error(StatusCode::NOT_FOUND));
        }
        Err(e) => {
            return Err(error_with_log_id(
                format!("unable to retrieve security code from database: {}", e),
                "Failed to delete security code",
                StatusCode::INTERNAL_SERVER_ERROR,
                log::Level::Error,
            ));
        }
    };

    let existing_code_data = DbState::into_inner(existing_code);

    // Check if user is a coordinator and if they have access to this project
    let is_coordinator = user.admin_role_id == AvailableAdminRole::Coordinator as i32;
    if is_coordinator {
        let is_assigned = coordinator_projects_repository::is_assigned(
            &data.db,
            user.admin_id,
            existing_code_data.project_id,
        )
        .await
        .map_err(|e| {
            error_with_log_id(
                format!("unable to check coordinator assignment: {}", e),
                "Failed to delete security code",
                StatusCode::INTERNAL_SERVER_ERROR,
                log::Level::Error,
            )
        })?;

        if !is_assigned {
            return Err("Access denied - you are not assigned to this project"
                .to_json_error(StatusCode::FORBIDDEN));
        }
    }

    // Delete the security code
    match delete_security_code(&data.db, security_code_id).await {
        Ok(_) => Ok(HttpResponse::Ok().json(DeleteCodeResponse {
            message: "Security code deleted successfully".to_string(),
        })),
        Err(e) => Err(error_with_log_id(
            format!("unable to delete security code from database: {}", e),
            "Failed to delete security code",
            StatusCode::INTERNAL_SERVER_ERROR,
            log::Level::Error,
        )),
    }
}
