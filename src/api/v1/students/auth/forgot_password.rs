use crate::app_data::AppData;
use crate::common::json_error::{error_with_log_id_and_payload, JsonError};
use crate::database::repositories::students_repository;
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
    /// The email address of the student account
    #[schema(example = "student@studenti.unitn.it")]
    email: String,
}

/// Requests a password reset for a student account
///
/// This endpoint sends a password reset email to the specified email address if a student
/// account with that email exists. The email contains a secure link to reset the password.
#[utoipa::path(
    post,
    path = "/v1/students/auth/forgot-password",
    request_body = ForgotPasswordSchema,
    responses(
        (status = 204, description = "Password reset email sent successfully (or email doesn't exist)"),
        (status = 500, description = "Internal server error", body = JsonError)
    ),
    tag = "Student authentication"
)]
pub(crate) async fn forgot_password_handler(
    body: Json<ForgotPasswordSchema>, 
    data: Data<AppData>,
) -> Result<HttpResponse, JsonError> {
    // Fetch the student by email
    let student_state = students_repository::get_by_email(&data.db, &body.email)
        .await
        .map_err(|e| {
            error_with_log_id_and_payload(
                format!("unable to fetch student from database: {}", e),
                "Password reset request failed",
                StatusCode::INTERNAL_SERVER_ERROR,
                log::Level::Error,
                &body,
            )
        })?;

    // If the student doesn't exist, still return success to prevent email enumeration
    // but don't actually send an email
    if let Some(student_state) = student_state {
        let student = DbState::into_inner(student_state);

        // Generate a secure token for password reset
        let token = generate_token(
            student.email.clone(),
            data.config.email_token_secret().clone(),
        )
        .map_err(|e| {
            error_with_log_id_and_payload(
                format!("unable to generate password reset token: {}", e),
                "Password reset request failed",
                StatusCode::INTERNAL_SERVER_ERROR,
                log::Level::Error,
                &body,
            )
        })?;

        // Create the reset URL with the token (frontend URL)
        let reset_url = format!(
            "{}/password-reset?t={}",
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
                    &body,
                ));
            }
        };

        // Send the password reset email
        let student_name = format!("{} {}", student.first_name, student.last_name);
        if let Err(e) = mailer
            .send_password_reset(student.email, student_name, &reset_url)
            .await
        {
            error!("failed to send password reset email: {}", e);
            return Err(error_with_log_id_and_payload(
                format!("unable to send password reset email: {}", e),
                "Password reset request failed",
                StatusCode::INTERNAL_SERVER_ERROR,
                log::Level::Error,
                &body,
            ));
        }
    }

    // Always return success to prevent email enumeration
    Ok(HttpResponse::NoContent().finish())
}
