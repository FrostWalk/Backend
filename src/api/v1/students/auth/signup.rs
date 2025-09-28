use crate::app_data::AppData;
use crate::common::json_error::{database_error, JsonError, ToJsonError};
use crate::mail::Mailer;
use crate::models::student::Student;
use actix_web::http::StatusCode;
use actix_web::web::{Data, Json};
use actix_web::HttpResponse;
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
    path = "/v1/students/auth/signup",
    request_body = StudentSignupScheme,
    responses(
        (status = 202, description = "Account created successfully", body = StudentSignupResponse),
        (status = 400, description = "Invalid data in request", body = JsonError),
        (status = 500, description = "Internal server error occurred", body = JsonError),
        (status = 503, description = "Account created email was not sent", body = JsonError)
    ),
    tag = "Student authentication",
)]
/// Creates a new student account
///
/// This endpoint allows students to register to the app.
pub(super) async fn student_signup_handler(
    payload: Json<StudentSignupScheme>, data: Data<AppData>,
) -> Result<HttpResponse, JsonError> {
    let scheme = payload.into_inner();

    // Validate that all fields are not empty or default values
    if scheme.first_name.trim().is_empty() {
        return Err("First name cannot be empty".to_json_error(StatusCode::BAD_REQUEST));
    } else if scheme.last_name.trim().is_empty() {
        return Err("Last name cannot be empty".to_json_error(StatusCode::BAD_REQUEST));
    } else if scheme.email.trim().is_empty() {
        return Err("Email cannot be empty".to_json_error(StatusCode::BAD_REQUEST));
    } else if scheme.password.trim().is_empty() {
        return Err("Password cannot be empty".to_json_error(StatusCode::BAD_REQUEST));
    }

    // check that email domain is valid
    let email_domain = scheme.email.split('@').nth(1);
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

    let mut result = DbState::new_uncreated(Student {
        student_id: 0,
        first_name: scheme.first_name,
        last_name: scheme.last_name,
        email: scheme.email,
        university_id: scheme.university_id,
        password_hash: generate_hash(scheme.password),
        is_pending: true,
    });

    if let Err(e) = result.save(&data.db).await {
        error!("unable to create student's account: {}", e);
        return Err(database_error());
    }

    let mailer = match Mailer::from_config(&data.config) {
        Ok(m) => m,
        Err(e) => {
            error!("unable to create instance of Mailer: {}", e);
            return Err(
                "Error sending confirmation email".to_json_error(StatusCode::INTERNAL_SERVER_ERROR)
            );
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
        error!("failed to send confirmation email: {}", e);
        return Err(
            "The account has been created but the confirmation email could not be sent; \
        ask the coordinator to approve you manually."
                .to_json_error(StatusCode::SERVICE_UNAVAILABLE),
        );
    }

    info!("new student account created: {:?}", result);

    Ok(HttpResponse::Ok().json(StudentSignupResponse {
        student_id: result.student_id,
    }))
}
