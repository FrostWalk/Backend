use crate::app_data::AppData;
use crate::common::json_error::{error_with_log_id_and_payload, JsonError};
use crate::database::repositories::admins_repository;
use crate::mail::Mailer;
use actix_web::http::StatusCode;
use actix_web::web::{Data, Json};
use actix_web::HttpResponse;
use confirm_email::generate_token;
use log::error;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use welds::state::DbState;

/// Request body for requesting a password reset
#[derive(Deserialize, Serialize, ToSchema)]
pub(crate) struct ForgotPasswordSchema {
    /// The email address of the admin account
    #[schema(example = "admin@unitn.it")]
    email: String,
}

/// Requests a password reset for an admin account
///
/// This endpoint sends a password reset email to the specified email address if an admin
/// account with that email exists. The email contains a secure link to reset the password.
#[utoipa::path(
    post,
    path = "/v1/admins/auth/forgot-password",
    request_body = ForgotPasswordSchema,
    responses(
        (status = 204, description = "Password reset email sent successfully or email doesn't exist"),
        (status = 500, description = "Internal server error", body = JsonError)
    ),
    tag = "Admin authentication"
)]
pub(crate) async fn forgot_password_handler(
    req: Json<ForgotPasswordSchema>, data: Data<AppData>,
) -> Result<HttpResponse, JsonError> {
    // Fetch the admin by email
    let admin_state = admins_repository::get_by_email(&data.db, &req.email)
        .await
        .map_err(|e| {
            error_with_log_id_and_payload(
                format!("unable to fetch admin from database: {}", e),
                "Password reset request failed",
                StatusCode::INTERNAL_SERVER_ERROR,
                log::Level::Error,
                &req,
            )
        })?;

    // If the admin doesn't exist, still return success to prevent email enumeration
    // but don't actually send an email
    if let Some(admin_state) = admin_state {
        let admin = DbState::into_inner(admin_state);

        // Generate a secure token for password reset
        let token = generate_token(
            admin.email.clone(),
            data.config.email_token_secret().clone(),
        )
        .map_err(|e| {
            error_with_log_id_and_payload(
                format!("unable to generate password reset token: {}", e),
                "Password reset request failed",
                StatusCode::INTERNAL_SERVER_ERROR,
                log::Level::Error,
                &req,
            )
        })?;

        // Create the reset URL with the token (frontend URL)
        let reset_url = format!(
            "{}/admin/password-reset?t={}",
            data.config.frontend_base_url(),
            token
        );

        // Create mailer instance
        let mailer = match Mailer::from_config(&data.config) {
            Ok(m) => m,
            Err(e) => {
                return Err(error_with_log_id_and_payload(
                    format!("unable to create instance of Mailer: {}", e),
                    "Password reset request failed",
                    StatusCode::INTERNAL_SERVER_ERROR,
                    log::Level::Error,
                    &req,
                ));
            }
        };

        // Send the password reset email
        let admin_name = format!("{} {}", admin.first_name, admin.last_name);
        if let Err(e) = mailer
            .send_password_reset(admin.email, admin_name, &reset_url)
            .await
        {
            error!("failed to send password reset email: {}", e);
            return Err(error_with_log_id_and_payload(
                format!("unable to send password reset email: {}", e),
                "Password reset request failed",
                StatusCode::INTERNAL_SERVER_ERROR,
                log::Level::Error,
                &req,
            ));
        }
    }

    // Always return success to prevent email enumeration
    Ok(HttpResponse::NoContent().finish())
}
