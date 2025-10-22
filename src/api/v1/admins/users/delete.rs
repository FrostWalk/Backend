use crate::app_data::AppData;
use crate::common::json_error::{error_with_log_id, JsonError, ToJsonError};
use crate::database::repositories::admins_repository;
use crate::jwt::get_user::LoggedUser;
use crate::models::admin_role::AvailableAdminRole;
use actix_web::http::StatusCode;
use actix_web::web::{Data, Path};
use actix_web::{HttpMessage, HttpRequest, HttpResponse};
use log::warn;

#[utoipa::path(
    delete,
    path = "/v1/admins/users/{id}",
    responses(
        (status = 200, description = "Admin deleted successfully"),
        (status = 404, description = "Admin not found", body = JsonError),
        (status = 500, description = "Internal server error occurred", body = JsonError)
    ),
    security(("AdminAuth" = [])),
    tag = "Admin users management",
)]
/// Delete an admin
#[actix_web_grants::protect(any("ROLE_ADMIN_ROOT", "ROLE_ADMIN_PROFESSOR"))]
pub(super) async fn delete_admin_handler(
    req: HttpRequest, path: Path<i32>, data: Data<AppData>,
) -> Result<HttpResponse, JsonError> {
    let admin_id = path.into_inner();

    // current user from request
    let user = match req.extensions().get_admin() {
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

    // Load the admin to delete
    let admin_state = admins_repository::get_by_id(&data.db, admin_id)
        .await
        .map_err(|e| {
            error_with_log_id(
                format!("unable to retrieve admin from database: {}", e),
                "Failed to delete user",
                StatusCode::INTERNAL_SERVER_ERROR,
                log::Level::Error,
            )
        })?;

    let admin_state = match admin_state {
        Some(s) => s,
        None => return Err("Admin not found".to_json_error(StatusCode::NOT_FOUND)),
    };

    // Only root can delete root users
    if (user.admin_role_id != AvailableAdminRole::Root as i32)
        && (admin_state.admin_role_id == AvailableAdminRole::Root as i32)
    {
        warn!("user {} tried to delete a root user", user.email);
        return Err("Operation not permitted".to_json_error(StatusCode::FORBIDDEN));
    }

    // Delete admin using repository function
    admins_repository::delete_by_id(&data.db, admin_id)
        .await
        .map_err(|e| {
            error_with_log_id(
                format!("unable to delete admin from database: {}", e),
                "Failed to delete user",
                StatusCode::INTERNAL_SERVER_ERROR,
                log::Level::Error,
            )
        })?;

    Ok(HttpResponse::Ok().finish())
}
