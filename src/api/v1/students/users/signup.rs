use crate::app_data::AppData;
use crate::common::json_error::{database_error, JsonError, ToJsonError};
use crate::mail::Mailer;
use crate::models::student::Student;
use actix_web::http::StatusCode;
use actix_web::web::{Data, Json};
use actix_web::{HttpRequest, HttpResponse};
use log::{error, info};
use password_auth::generate_hash;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use welds::state::DbState;

#[derive(Debug, Deserialize, ToSchema)]
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
    path = "/v1/students/signup",
    request_body = StudentSignupScheme,
    responses(
        (status = 202, description = "Account created successfully", body = StudentSignupResponse),
        (status = 400, description = "Invalid data in request", body = JsonError),
        (status = 500, description = "Internal server error occurred", body = JsonError)
    ),
    tag = "Student users management",
)]
/// Creates a new student account
///
/// This endpoint allows students to register to the app.
pub(super) async fn student_signup_handler(payload: Json<StudentSignupScheme>, data: Data<AppData>,
) -> Result<HttpResponse, JsonError> {
    let scheme = payload.into_inner();

    // Validate that all fields are not empty or default values
    if scheme.first_name.trim().is_empty() {
        return Err("first name cannot be empty".to_json_error(StatusCode::BAD_REQUEST));
    } else if scheme.last_name.trim().is_empty() {
        return Err("last name cannot be empty".to_json_error(StatusCode::BAD_REQUEST));
    } else if scheme.email.trim().is_empty() {
        return Err("email cannot be empty".to_json_error(StatusCode::BAD_REQUEST));
    } else if scheme.password.trim().is_empty() {
        return Err("password cannot be empty".to_json_error(StatusCode::BAD_REQUEST));
    }

    // check that email domain is valid
    let email_domain = scheme.email.split('@').nth(1);
    if let Some(domain) = email_domain {
        let allowed_domains = data.config.allowed_signup_domains();
        if !allowed_domains.contains(&domain.to_string()) {
            return Err(
                "email domain not allowed for signup".to_json_error(StatusCode::BAD_REQUEST)
            );
        }
    } else {
        return Err("invalid email format".to_json_error(StatusCode::BAD_REQUEST));
    }

    let mut state = DbState::new_uncreated(Student {
        student_id: 0,
        first_name: scheme.first_name,
        last_name: scheme.last_name,
        email: scheme.email,
        university_id: scheme.university_id,
        password_hash: generate_hash(scheme.password),
        is_pending: false,
    });

    if let Err(e) = state.save(&data.db).await {
        error!("unable to create student's account: {}", e);
        return Err(database_error());
    }

    // todo finishing email sending
    // let mailer = Mailer::new();
    //
    // if let Err(e) = Mailer::send_account_confirmation(
    //     &scheme.email,
    //     scheme.first_name,
    //     &format!("student_{}_confirmation", state.student_id),
    // ).await {
    //     error!("failed to send confirmation email: {}", e);
    // }

    info!("New student's account created: {:?}", state);

    Ok(HttpResponse::Ok().json(StudentSignupResponse {
        student_id: state.student_id,
    }))
}
