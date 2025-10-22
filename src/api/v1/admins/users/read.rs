use crate::api::v1::admins::users::AdminResponseScheme;
use crate::app_data::AppData;
use crate::common::json_error::{error_with_log_id, JsonError, ToJsonError};
use crate::database::repositories::admins_repository;
use actix_web::http::StatusCode;
use actix_web::web::{Data, Path};
use actix_web::HttpResponse;
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
#[actix_web_grants::protect(any("ROLE_ADMIN_ROOT", "ROLE_ADMIN_PROFESSOR"))]
pub(super) async fn get_all_admins_handler(data: Data<AppData>) -> Result<HttpResponse, JsonError> {
    let states = admins_repository::get_all(&data.db).await.map_err(|e| {
        error_with_log_id(
            format!("unable to retrieve admins from database: {}", e),
            "Failed to retrieve users",
            StatusCode::INTERNAL_SERVER_ERROR,
            log::Level::Error,
        )
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
#[actix_web_grants::protect(any("ROLE_ADMIN_ROOT", "ROLE_ADMIN_PROFESSOR"))]
pub(super) async fn get_one_admin_handler(
    path: Path<i32>, data: Data<AppData>,
) -> Result<HttpResponse, JsonError> {
    let id = path.into_inner();

    let admin_state = admins_repository::get_by_id(&data.db, id)
        .await
        .map_err(|e| {
            error_with_log_id(
                format!("unable to retrieve admin from database: {}", e),
                "Failed to retrieve users",
                StatusCode::INTERNAL_SERVER_ERROR,
                log::Level::Error,
            )
        })?;

    let state = match admin_state {
        Some(a) => a,
        None => return Err("Admin not found".to_json_error(StatusCode::NOT_FOUND)),
    };

    let admin = AdminResponseScheme::from(DbState::into_inner(state));

    Ok(HttpResponse::Ok().json(admin))
}
