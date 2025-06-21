use crate::api::v1::admins::users::AdminResponseScheme;
use crate::app_data::AppData;
use crate::common::json_error::{JsonError, ToJsonError};
use crate::database::repository_methods_trait::RepositoryMethods;
use actix_web::http::StatusCode;
use actix_web::web::Data;
use actix_web::{web, HttpResponse};
use log::error;
use serde::Serialize;
use utoipa::ToSchema;

#[derive(Debug, Serialize, ToSchema)]
pub(crate) struct GetAllAdminsResponse {
    pub admins: Vec<AdminResponseScheme>,
}
#[utoipa::path(
    get,
    path = "/v1/admins/users",
    responses(
        (status = 200, description = "Found admins", body = GetAllAdminsResponse),
        (status = 500, description = "Internal server error occurred", body = JsonError)
    ),
    security(("AdminAuth" = [])),
    tag = "Admin users management",
)]
/// Handler for retrieving a list of admin users
///
/// Returns array with all the data of the admins except passwords
pub(super) async fn get_all_admins_handler(data: Data<AppData>) -> Result<HttpResponse, JsonError> {
    let found = data.repositories.admins.get_all().await;

    let admins: Vec<AdminResponseScheme> = match found {
        Ok(a) => a.into_iter().map(AdminResponseScheme::from).collect(),
        Err(e) => {
            error!("unable to retrieve admins from database: {}", e);
            return Err("database error".to_json_error(StatusCode::INTERNAL_SERVER_ERROR));
        }
    };

    Ok(HttpResponse::Ok().json(GetAllAdminsResponse { admins }))
}
#[utoipa::path(
    get,
    path = "/v1/admins/users/{id}",
    responses(
        (status = 200, description = "Found admin", body = AdminResponseScheme),
        (status = 404, description = "Admin not found", body = JsonError),
        (status = 500, description = "Internal server error", body = JsonError)
    ),
    security(("AdminAuth" = [])),
    tag = "Admin users management",
)]
/// Handler for retrieving a single admin user by ID
///
/// Returns detailed information about a specific admin user
/// without including sensitive fields like passwords.
pub(super) async fn get_one_admin_handler(
    path: web::Path<i32>, data: Data<AppData>,
) -> Result<HttpResponse, JsonError> {
    let id = path.into_inner();

    let found = match data.repositories.admins.get_from_id(id).await {
        Ok(a) => a,
        Err(e) => {
            error!("unable to retrieve admin from database: {}", e);
            return Err("database error".to_json_error(StatusCode::INTERNAL_SERVER_ERROR));
        }
    };

    let admin = match found {
        None => {
            return Err("admin not found".to_json_error(StatusCode::NOT_FOUND));
        }
        Some(u) => AdminResponseScheme::from(u),
    };

    Ok(HttpResponse::Ok().json(admin))
}
