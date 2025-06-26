use crate::app_data::AppData;
use crate::common::json_error::JsonError;
use actix_web::web::{Data, Json};
use actix_web::HttpResponse;
use serde::Deserialize;
use utoipa::ToSchema;

#[derive(Debug, Deserialize, ToSchema)]
pub(super) struct StudentSignupScheme {
    #[schema(example = "John")]
    first_name: String,
    #[schema(example = "Doe")]
    last_name: String,
    #[schema(example = "john.doe@example.com")]
    email: String,
    #[schema(example = "123456")]
    university_id: String,
    #[schema(example = "SecureP@ssw0rd")]
    password: String,
}

#[utoipa::path(
    post,
    path = "/v1/students/users/signup",
    request_body = StudentSignupScheme,
    responses(
        (status = 200, description = "Successfully registered"),
        (status = 400, description = "Missing data", body = JsonError),
        (status = 500, description = "Internal server error during serialization or database query", body = JsonError)
    ),
    tag = "Student users management",
)]
/// Register a new student account
pub(super) async fn student_signup_handler(
    payload: Json<StudentSignupScheme>, data: Data<AppData>,
) -> Result<HttpResponse, JsonError> {
    //todo endpoint logic
    Ok(HttpResponse::Ok().finish())
}
