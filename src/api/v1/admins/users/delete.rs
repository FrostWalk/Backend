use crate::app_data::AppData;
use crate::common::json_error::{database_error, JsonError, ToJsonError};
use crate::jwt::get_user::LoggedUser;
use crate::models::admin::Admin;
use crate::models::admin_role::AvailableAdminRole;
use actix_web::http::StatusCode;
use actix_web::web::Data;
use actix_web::{web, HttpMessage, HttpRequest, HttpResponse};
use log::{error, warn};

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
pub(super) async fn delete_admin_handler(
    req: HttpRequest, path: web::Path<i32>, data: Data<AppData>,
) -> Result<HttpResponse, JsonError> {
    let admin_id = path.into_inner();

    // current user from request
    let user = match req.extensions().get_admin() {
        Ok(user) => user,
        Err(e) => {
            error!("entered a protected route without a user loaded in the request");
            return Err(e.to_json_error(StatusCode::INTERNAL_SERVER_ERROR));
        }
    };

    // Load the admin to delete
    let mut rows = Admin::where_col(|a| a.admin_id.equal(admin_id))
        .run(&data.db)
        .await
        .map_err(|e| {
            error!("unable to retrieve admin from database: {}", e);
            database_error()
        })?;

    let mut admin_state = match rows.pop() {
        Some(s) => s,
        None => return Err("admin not found".to_json_error(StatusCode::NOT_FOUND)),
    };

    // Only root can delete root users
    if (user.admin_role_id != AvailableAdminRole::Root as i32)
        && (admin_state.admin_role_id == AvailableAdminRole::Root as i32)
    {
        warn!("The user {} tried to delete a root user", user.email);
        return Err("operation not permitted".to_json_error(StatusCode::FORBIDDEN));
    }

    admin_state.delete(&data.db).await.map_err(|e| {
        error!("unable to delete admin from database: {}", e);
        database_error()
    })?;

    Ok(HttpResponse::Ok().finish())
}
