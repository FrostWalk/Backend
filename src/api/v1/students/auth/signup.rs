use crate::app_data::AppData;
use crate::common::json_error::{error_with_log_id_and_payload, JsonError, ToJsonError};
use crate::database::repositories::students_repository;
use crate::logging::payload_capture::capture_response_status;
use crate::mail::Mailer;
use crate::models::student::Student;
use actix_web::http::StatusCode;
use actix_web::web::{Data, Json};
use actix_web::HttpResponse;
use log::info;
use password_auth::generate_hash;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use welds::state::DbState;

#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub(crate) struct StudentSignupScheme {
    #[schema(example = "John")]
    pub first_name: String,
    #[schema(example = "Doe")]
    pub last_name: String,
    #[schema(example = "john.doe@studenti.unitn.it")]
    pub email: String,
    #[schema(example = "SecureP@ss123")]
    pub password: String,
    #[schema(example = "123456")]
    pub university_id: i32,
}

#[derive(Debug, Serialize, ToSchema)]
pub(crate) struct StudentSignupResponse {
    #[schema(example = "123")]
    pub student_id: i32,
}

#[utoipa::path(
    post,
    path = "/v1/students/auth/signup",
    request_body = StudentSignupScheme,
    responses(
        (status = 202, description = "Account created successfully", body = StudentSignupResponse),
        (status = 400, description = "Invalid data in request", body = JsonError),
        (status = 409, description = "Student with this email or university ID already exists", body = JsonError),
        (status = 500, description = "Internal server error occurred", body = JsonError),
        (status = 503, description = "Account created email was not sent", body = JsonError)
    ),
    tag = "Student authentication",
)]
/// Creates a new student account
///
/// This endpoint allows students to register to the app.
pub(super) async fn student_signup_handler(
    req: Json<StudentSignupScheme>, data: Data<AppData>,
) -> Result<HttpResponse, JsonError> {
    // Validate that all fields are not empty or default values
    if req.first_name.trim().is_empty() {
        return Err("First name cannot be empty".to_json_error(StatusCode::BAD_REQUEST));
    } else if req.last_name.trim().is_empty() {
        return Err("Last name cannot be empty".to_json_error(StatusCode::BAD_REQUEST));
    } else if req.email.trim().is_empty() {
        return Err("Email cannot be empty".to_json_error(StatusCode::BAD_REQUEST));
    } else if req.password.trim().is_empty() {
        return Err("Password cannot be empty".to_json_error(StatusCode::BAD_REQUEST));
    }

    // check that email domain is valid
    let email_domain = req.email.split('@').nth(1);
    if let Some(domain) = email_domain {
        let allowed_domains = data.config.allowed_signup_domains();
        if !allowed_domains.contains(&domain.to_string()) {
            return Err(
                "Email domain not allowed for signup".to_json_error(StatusCode::BAD_REQUEST)
            );
        }
    } else {
        return Err("Invalid email format".to_json_error(StatusCode::BAD_REQUEST));
    }

    // Check if email already exists
    let email_exists = students_repository::email_exists(&data.db, &req.email)
        .await
        .map_err(|e| {
            error_with_log_id_and_payload(
                format!("unable to check if email exists: {}", e),
                "Account creation failed",
                StatusCode::INTERNAL_SERVER_ERROR,
                log::Level::Error,
                &req,
            )
        })?;

    if email_exists {
        return Err("User with this email already exists".to_json_error(StatusCode::CONFLICT));
    }

    // Check if university ID already exists
    let university_id_exists =
        students_repository::university_id_exists(&data.db, req.university_id)
            .await
            .map_err(|e| {
                error_with_log_id_and_payload(
                    format!("unable to check if university ID exists: {}", e),
                    "Account creation failed",
                    StatusCode::INTERNAL_SERVER_ERROR,
                    log::Level::Error,
                    &req,
                )
            })?;

    if university_id_exists {
        return Err(
            "User with this university ID already exists".to_json_error(StatusCode::CONFLICT)
        );
    }

    let mut result = DbState::new_uncreated(Student {
        student_id: 0,
        first_name: req.first_name.clone(),
        last_name: req.last_name.clone(),
        email: req.email.clone(),
        university_id: req.university_id,
        password_hash: generate_hash(req.password.clone()),
        is_pending: true,
    });

    if let Err(e) = result.save(&data.db).await {
        return Err(error_with_log_id_and_payload(
            format!("unable to create student's account: {}", e),
            "Account creation failed",
            StatusCode::INTERNAL_SERVER_ERROR,
            log::Level::Error,
            &req,
        ));
    }

    let mailer = match Mailer::from_config(&data.config) {
        Ok(m) => m,
        Err(e) => {
            return Err(error_with_log_id_and_payload(
                format!("unable to create instance of Mailer: {}", e),
                "Account creation failed",
                StatusCode::INTERNAL_SERVER_ERROR,
                log::Level::Error,
                &req,
            ));
        }
    };

    let name = format!("{} {}", &result.first_name, &result.last_name);
    if let Err(e) = mailer
        .send_account_confirmation(
            result.email.clone(),
            name,
            data.config.email_token_secret().clone(),
        )
        .await
    {
        return Err(error_with_log_id_and_payload(
            format!("failed to send confirmation email: {}", e),
            "Account created but confirmation email could not be sent",
            StatusCode::SERVICE_UNAVAILABLE,
            log::Level::Error,
            &req,
        ));
    }

    info!("new student account created: {:?}", result);

    // Capture successful response status
    capture_response_status(200);

    Ok(HttpResponse::Ok().json(StudentSignupResponse {
        student_id: result.student_id,
    }))
}
