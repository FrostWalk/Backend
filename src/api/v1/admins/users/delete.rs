use crate::app_data::AppData;
use crate::common::json_error::{JsonError, ToJsonError};
use crate::database::repositories::admins_repository::AdminRole;
use crate::database::repository_methods_trait::RepositoryMethods;
use crate::jwt::get_user::LoggedUser;
use actix_web::http::StatusCode;
use actix_web::web::Data;
use actix_web::{web, HttpMessage, HttpRequest, HttpResponse};
use log::{error, warn};
use sea_orm::{DbErr, DeleteResult};

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
pub(super) async fn delete_admin_handler(
    req: HttpRequest, path: web::Path<i32>, data: Data<AppData>,
) -> Result<HttpResponse, JsonError> {
    let admin_id = path.into_inner();

    let user = match req.extensions().get_admin() {
        Ok(user) => user,
        Err(e) => {
            error!("entered a protected route without a user loaded in the request");
            return Err(e.to_json_error(StatusCode::INTERNAL_SERVER_ERROR));
        }
    };

    // Check if admin exists
    let admin = match data.repositories.admins.get_from_id(admin_id).await {
        Ok(a) => match a {
            None => {
                return Err("admin not found".to_json_error(StatusCode::NOT_FOUND));
            }
            Some(a) => a,
        },
        Err(e) => {
            error!("unable to retrieve admin from database {}", e);
            return Err("database error ".to_json_error(StatusCode::INTERNAL_SERVER_ERROR));
        }
    };

    // only root can delete root users
    if (user.admin_role_id != AdminRole::Root as i32)
        && (admin.admin_role_id == AdminRole::Root as i32)
    {
        warn!("The user {} tried to delete a root user", user.email);
        return Err("operation not permitted".to_json_error(StatusCode::FORBIDDEN));
    }

    match data.repositories.admins.delete_from_id(admin_id).await {
        Ok(_) => {}
        Err(e) => {
            error!("unable to delete admin from database {}", e);
            return Err("database error ".to_json_error(StatusCode::INTERNAL_SERVER_ERROR));
        }
    };

    Ok(HttpResponse::Ok().finish())
}
