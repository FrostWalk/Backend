use crate::app_data::AppData;
use crate::common::json_error::{error_with_log_id_and_payload, JsonError, ToJsonError};
use crate::jwt::get_user::LoggedUser;
use actix_web::http::StatusCode;
use actix_web::web::{Data, Json};
use actix_web::{HttpMessage, HttpRequest, HttpResponse};
use log::error;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub(crate) struct TestEmailRequest {
    #[schema(example = "test@example.com", format = "email")]
    pub to_email: String,
    #[schema(example = "Test Email Subject")]
    pub subject: String,
    #[schema(example = "This is a test email body")]
    pub body: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub(crate) struct TestEmailResponse {
    #[schema(example = "Email sent successfully")]
    pub message: String,
}

#[utoipa::path(
    post,
    path = "/v1/admins/users/test-email",
    request_body = TestEmailRequest,
    responses(
        (status = 200, description = "Test email sent successfully", body = TestEmailResponse),
        (status = 400, description = "Invalid email address or request data", body = JsonError),
        (status = 401, description = "Authentication required", body = JsonError),
        (status = 403, description = "Insufficient permissions - root access required", body = JsonError),
        (status = 500, description = "Failed to send email", body = JsonError)
    ),
    security(("AdminAuth" = [])),
    tag = "Admin users management",
)]
/// Sends a test email to verify SMTP configuration.
///
/// This endpoint allows root administrators to send test emails to verify that the mailer
/// is properly configured. The email is sent directly without using templates.
///
/// **Security**: Only users with ROLE_ADMIN_ROOT can access this endpoint.
#[actix_web_grants::protect("ROLE_ADMIN_ROOT")]
pub(super) async fn test_email_handler(
    req: HttpRequest, body: Json<TestEmailRequest>, data: Data<AppData>,
) -> Result<HttpResponse, JsonError> {
    let _user = match req.extensions().get_admin() {
        Ok(user) => user,
        Err(_e) => {
            return Err(error_with_log_id_and_payload(
                "entered a protected route without a user loaded in the request",
                "Authentication error",
                StatusCode::INTERNAL_SERVER_ERROR,
                log::Level::Error,
                &body,
            ));
        }
    };

    // Validate email address
    let to_email = body.to_email.trim();
    if to_email.is_empty() {
        return Err("Email address cannot be empty".to_json_error(StatusCode::BAD_REQUEST));
    }

    // Validate subject and body
    if body.subject.trim().is_empty() {
        return Err("Subject cannot be empty".to_json_error(StatusCode::BAD_REQUEST));
    }

    if body.body.trim().is_empty() {
        return Err("Body cannot be empty".to_json_error(StatusCode::BAD_REQUEST));
    }

    // Send test email using Mailer
    if let Err(e) = data
        .mailer
        .send_test_email(
            to_email.to_string(),
            body.subject.clone(),
            body.body.clone(),
        )
        .await
    {
        error!("Failed to send test email to {}: {}", to_email, e);
        return Err(error_with_log_id_and_payload(
            format!("Failed to send test email: {}", e),
            "Email sending failed",
            StatusCode::INTERNAL_SERVER_ERROR,
            log::Level::Error,
            &body,
        ));
    }

    Ok(HttpResponse::Ok().json(TestEmailResponse {
        message: format!("Test email sent successfully to {}", to_email),
    }))
}
