use crate::api::v1::admins::users::AdminResponseScheme;
use crate::app_data::AppData;
use crate::common::json_error::{database_error, JsonError, ToJsonError};
use crate::models::admin::Admin;
use actix_web::http::StatusCode;
use actix_web::web::Data;
use actix_web::{web, HttpResponse};
use log::error;
use serde::Serialize;
use utoipa::ToSchema;
use welds::state::DbState;

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
pub(super) async fn get_all_admins_handler(
    data: web::Data<AppData>,
) -> Result<HttpResponse, JsonError> {
    let states = Admin::all().run(&data.db).await.map_err(|e| {
        error!("unable to retrieve admins from database: {}", e);
        database_error()
    })?;

    let admins: Vec<AdminResponseScheme> = states
        .into_iter()
        .map(DbState::into_inner)
        .map(AdminResponseScheme::from)
        .collect();

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

    let mut rows = Admin::where_col(|a| a.admin_id.equal(id))
        .run(&data.db)
        .await
        .map_err(|e| {
            error!("unable to retrieve admin from database: {}", e);
            database_error()
        })?;

    let state = match rows.pop() {
        Some(a) => a,
        None => return Err("Admin not found".to_json_error(StatusCode::NOT_FOUND)),
    };

    let admin = AdminResponseScheme::from(DbState::into_inner(state));

    Ok(HttpResponse::Ok().json(admin))
}
