use crate::app_data::AppData;
use crate::common::json_error::{error_with_log_id, JsonError, ToJsonError};
use crate::database::repositories::students_repository;
use actix_web::http::StatusCode;
use actix_web::web::{Data, Query};
use actix_web::HttpResponse;
use confirm_email::validate_token;
use log::{error, info};
use serde::Deserialize;
use utoipa::ToSchema;

#[derive(Debug, Deserialize, ToSchema)]
pub(crate) struct ConfirmTokenQuery {
    #[schema(example = "eyJhbGciOiJIUzI1NiIsIn...")]
    pub t: String,
}

#[utoipa::path(
    get,
    path = "/v1/students/auth/confirm",
    params(
        ("t" = String, Query, description = "Email confirmation token")
    ),
    responses(
        (status = 204, description = "Account confirmed successfully"),
        (status = 400, description = "Invalid token", body = JsonError),
        (status = 500, description = "Internal server error occurred", body = JsonError)
    ),
    tag = "Student authentication",
)]
/// Confirms a student account using email verification token
///
/// This endpoint verifies the email confirmation token sent to the student's email
/// and activates their account by setting is_pending to false.
pub(super) async fn confirm_student_handler(
    query: Query<ConfirmTokenQuery>, 
    data: Data<AppData>,
) -> Result<HttpResponse, JsonError> {
    let token = &query.t;

    // Validate the token and extract the email
    let email = match validate_token(token.clone(), data.config.email_token_secret().clone()) {
        Ok(email) => email,
        Err(e) => {
            error!("invalid confirmation token: {}", e);
            return Err(
                "Invalid or expired confirmation token".to_json_error(StatusCode::BAD_REQUEST)
            );
        }
    };

    let student_state = students_repository::get_by_email(&data.db, &email)
        .await
        .map_err(|e| {
            error_with_log_id(
                format!("unable to fetch student from database: {}", e),
                "Account confirmation failed",
                StatusCode::INTERNAL_SERVER_ERROR,
                log::Level::Error,
            )
        })?;

    let student_state = match student_state {
        Some(student) => student,
        None => {
            error!("student with email {} not found", email);
            return Err("Student account not found".to_json_error(StatusCode::BAD_REQUEST));
        }
    };

    let mut student_state = student_state;
    student_state.is_pending = false;

    if let Err(e) = student_state.save(&data.db).await {
        return Err(error_with_log_id(
            format!("unable to update student confirmation status: {}", e),
            "Account confirmation failed",
            StatusCode::INTERNAL_SERVER_ERROR,
            log::Level::Error,
        ));
    }

    info!("student account confirmed successfully: {}", email);

    Ok(HttpResponse::NoContent().finish())
}
