use crate::app_data::AppData;
use crate::common::json_error::{error_with_log_id_and_payload, JsonError, ToJsonError};
use crate::database::repositories::admins_repository;
use actix_web::http::StatusCode;
use actix_web::web::{Data, Json, Path};
use actix_web::HttpResponse;
use password_auth::generate_hash;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub(crate) struct UpdateAdminScheme {
    #[schema(example = "John")]
    pub first_name: Option<String>,
    #[schema(example = "Doe")]
    pub last_name: Option<String>,
    #[schema(example = "john.doe@example.com")]
    pub email: Option<String>,
    #[schema(example = "SecureP@ss123")]
    pub password: Option<String>,
}
#[utoipa::path(
    patch,
    path = "/v1/admins/users/{id}",
    request_body = UpdateAdminScheme,
    responses(
        (status = 200, description = "Admin updated successfully"),
        (status = 400, description = "Invalid data in request", body = JsonError),
        (status = 401, description = "Authentication required", body = JsonError),
        (status = 500, description = "Internal server error occurred", body = JsonError)
    ),
    security(("AdminAuth" = [])),
    tag = "Admin users management",
)]
/// Updates an existing admin user.
///
/// This endpoint allows authenticated admins to update their own or other admin's details. Only root admins can modify roles.
#[actix_web_grants::protect(any("ROLE_ADMIN_ROOT", "ROLE_ADMIN_PROFESSOR"))]
pub(super) async fn update_admin_handler(
    path: Path<i32>, body: Json<UpdateAdminScheme>, data: Data<AppData>,
) -> Result<HttpResponse, JsonError> {
    let id = path.into_inner();

    // Check if admin exists
    let admin_exists = admins_repository::get_by_id(&data.db, id)
        .await
        .map_err(|e| {
            error_with_log_id_and_payload(
                format!("unable to load admin {}: {}", id, e),
                "Failed to update user",
                StatusCode::INTERNAL_SERVER_ERROR,
                log::Level::Error,
                &body,
            )
        })?
        .is_some();

    if !admin_exists {
        return Err("Admin not found".to_json_error(StatusCode::NOT_FOUND));
    }

    // Update admin using repository function
    let password_hash = body.password.as_ref().map(generate_hash);

    admins_repository::update_by_id(
        &data.db,
        id,
        body.first_name.clone(),
        body.last_name.clone(),
        body.email.clone(),
        password_hash,
    )
    .await
    .map_err(|e| {
        error_with_log_id_and_payload(
            format!("unable to update admin {}: {}", id, e),
            "Failed to update user",
            StatusCode::INTERNAL_SERVER_ERROR,
            log::Level::Error,
            &body,
        )
    })?;

    Ok(HttpResponse::Ok().finish())
}
