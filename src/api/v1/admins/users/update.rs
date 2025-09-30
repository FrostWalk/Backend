use crate::app_data::AppData;
use crate::common::json_error::{error_with_log_id_and_payload, JsonError, ToJsonError};
use crate::database::repositories::admins_repository;
use actix_web::http::StatusCode;
use actix_web::web::Json;
use actix_web::{web, HttpResponse};
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
pub(super) async fn update_admin_handler(
    path: web::Path<i32>, req: Json<UpdateAdminScheme>, data: web::Data<AppData>,
) -> Result<HttpResponse, JsonError> {
    let id = path.into_inner();

    let admin_state_opt = admins_repository::get_by_id(&data.db, id)
        .await
        .map_err(|e| {
            error_with_log_id_and_payload(
                format!("unable to load admin {}: {}", id, e),
                "Failed to update user",
                StatusCode::INTERNAL_SERVER_ERROR,
                log::Level::Error,
                &req,
            )
        })?;

    let mut admin_state = match admin_state_opt {
        Some(s) => s,
        None => return Err("Admin not found".to_json_error(StatusCode::NOT_FOUND)),
    };

    // Apply only provided fields
    if let Some(v) = req.first_name.clone() {
        admin_state.first_name = v;
    }
    if let Some(v) = req.last_name.clone() {
        admin_state.last_name = v;
    }
    if let Some(v) = req.email.clone() {
        admin_state.email = v;
    }
    if let Some(v) = req.password.clone() {
        admin_state.password_hash = generate_hash(v);
    }

    admin_state.save(&data.db).await.map_err(|e| {
        error_with_log_id_and_payload(
            format!("unable to update admin {}: {}", id, e),
            "Failed to update user",
            StatusCode::INTERNAL_SERVER_ERROR,
            log::Level::Error,
            &req,
        )
    })?;

    Ok(HttpResponse::Ok().finish())
}
