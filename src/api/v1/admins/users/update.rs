use crate::app_data::AppData;
use crate::common::json_error::{JsonError, ToJsonError};
use crate::database::repository_methods_trait::RepositoryMethods;
use actix_web::http::StatusCode;
use actix_web::web::{Data, Json};
use actix_web::HttpResponse;
use entity::admins;
use log::error;
use sea_orm::ActiveValue;
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
    path = "/v1/admins/users",
    request_body = UpdateAdminScheme,
    responses(
        (status = 200, description = "Admin updated successfully"),
        (status = 400, description = "Invalid data in request", body = JsonError),
        (status = 401, description = "Authentication required", body = JsonError),
        (status = 500, description = "Internal server error occurred", body = JsonError)
    ),
    tag = "Admin users management",
)]
/// Updates an existing admin user.
///
/// This endpoint allows authenticated admins to update their own or other admin's details. Only root admins can modify roles.
pub(super) async fn update_admin_handler(
    payload: Json<UpdateAdminScheme>, data: Data<AppData>,
) -> Result<HttpResponse, JsonError> {
    let scheme = payload.into_inner();

    let admin_update = admins::ActiveModel {
        admin_id: ActiveValue::NotSet,
        first_name: scheme
            .first_name
            .map_or(ActiveValue::NotSet, ActiveValue::Set),
        last_name: scheme
            .last_name
            .map_or(ActiveValue::NotSet, ActiveValue::Set),
        email: scheme.email.map_or(ActiveValue::NotSet, ActiveValue::Set),
        password_hash: scheme.password.map_or(ActiveValue::NotSet, |v| {
            let hashed = password_auth::generate_hash(v);
            ActiveValue::Set(hashed)
        }),
        admin_role_id: ActiveValue::NotSet,
    };

    data.repositories
        .admins
        .update(admin_update)
        .await
        .map_err(|e| {
            error!("Unable to update admin: {}", e);
            "Unable to update admin scheme".to_json_error(StatusCode::INTERNAL_SERVER_ERROR)
        })?;

    Ok(HttpResponse::Ok().finish())
}
