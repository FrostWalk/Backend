use crate::app_data::AppData;
use crate::common::json_error::{JsonError, ToJsonError};
use crate::models::admin::Admin;
use actix_web::http::StatusCode;
use actix_web::web::Json;
use actix_web::{web, HttpResponse};
use log::error;
use password_auth::generate_hash;
use serde::Deserialize;
use utoipa::ToSchema;

#[derive(Debug, Deserialize, ToSchema)]
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
    path: web::Path<i32>, payload: Json<UpdateAdminScheme>, data: web::Data<AppData>,
) -> Result<HttpResponse, JsonError> {
    let id = path.into_inner();
    let scheme = payload.into_inner();

    let mut rows = Admin::where_col(|a| a.admin_id.equal(id))
        .run(&data.db)
        .await
        .map_err(|e| {
            error!("unable to load admin {}: {}", id, e);
            "Unable to load admin".to_json_error(StatusCode::INTERNAL_SERVER_ERROR)
        })?;

    let mut admin_state = match rows.pop() {
        Some(s) => s,
        None => return Err("Admin not found".to_json_error(StatusCode::NOT_FOUND)),
    };

    // Apply only provided fields
    if let Some(v) = scheme.first_name {
        admin_state.first_name = v;
    }
    if let Some(v) = scheme.last_name {
        admin_state.last_name = v;
    }
    if let Some(v) = scheme.email {
        admin_state.email = v;
    }
    if let Some(v) = scheme.password {
        admin_state.password_hash = generate_hash(v);
    }

    admin_state.save(&data.db).await.map_err(|e| {
        error!("unable to update admin {}: {}", id, e);
        "Unable to update admin".to_json_error(StatusCode::INTERNAL_SERVER_ERROR)
    })?;

    Ok(HttpResponse::Ok().finish())
}
