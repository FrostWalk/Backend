use crate::app_data::AppData;
use crate::common::json_error::{error_with_log_id_and_payload, JsonError, ToJsonError};
use crate::database::repositories::admins_repository;
use actix_web::http::StatusCode;
use actix_web::web::{Data, Json, Query};
use actix_web::HttpResponse;
use confirm_email::validate_token;
use log::{error, info};
use password_auth::generate_hash;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

/// Query parameter for the password reset token
#[derive(Debug, Deserialize, ToSchema)]
pub(crate) struct ResetPasswordQuery {
    /// The password reset token sent via email
    #[schema(example = "eyJhbGciOiJIUzI1NiIsIn...")]
    pub t: String,
}

/// Request body for resetting a password
#[derive(Deserialize, Serialize, ToSchema)]
pub(crate) struct ResetPasswordSchema {
    /// The new password for the admin account
    #[schema(example = "newSecurePassword123!")]
    new_password: String,
}

/// Resets an admin's password using a valid reset token
///
/// This endpoint validates the password reset token and updates the admin's password.
/// The token is sent to the admin's email via the forgot-password endpoint.
#[utoipa::path(
    post,
    path = "/v1/admins/auth/reset-password",
    params(
        ("t" = String, Query, description = "Password reset token from email")
    ),
    request_body = ResetPasswordSchema,
    responses(
        (status = 204, description = "Password reset successfully"),
        (status = 400, description = "Invalid or expired token", body = JsonError),
        (status = 500, description = "Internal server error", body = JsonError)
    ),
    tag = "Admin authentication"
)]
pub(crate) async fn reset_password_handler(
    query: Query<ResetPasswordQuery>, req: Json<ResetPasswordSchema>, data: Data<AppData>,
) -> Result<HttpResponse, JsonError> {
    let token = &query.t;

    // Validate the token and extract the email
    let email = match validate_token(token.clone(), data.config.email_token_secret().clone()) {
        Ok(email) => email,
        Err(e) => {
            error!("invalid password reset token: {}", e);
            return Err(
                "Invalid or expired password reset token".to_json_error(StatusCode::BAD_REQUEST)
            );
        }
    };

    // Fetch the admin by email
    let admin_state = admins_repository::get_by_email(&data.db, &email)
        .await
        .map_err(|e| {
            error_with_log_id_and_payload(
                format!("unable to fetch admin from database: {}", e),
                "Password reset failed",
                StatusCode::INTERNAL_SERVER_ERROR,
                log::Level::Error,
                &req,
            )
        })?;

    let admin_state = match admin_state {
        Some(admin) => admin,
        None => {
            error!("admin with email {} not found", email);
            return Err("Admin account not found".to_json_error(StatusCode::BAD_REQUEST));
        }
    };

    // Update the password hash
    let mut admin_state = admin_state;
    admin_state.password_hash = generate_hash(&req.new_password);

    if let Err(e) = admin_state.save(&data.db).await {
        return Err(error_with_log_id_and_payload(
            format!("unable to update admin password: {}", e),
            "Password reset failed",
            StatusCode::INTERNAL_SERVER_ERROR,
            log::Level::Error,
            &req,
        ));
    }

    info!("admin password reset successfully: {}", email);

    Ok(HttpResponse::NoContent().finish())
}
