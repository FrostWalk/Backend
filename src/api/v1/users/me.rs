use crate::app_data::AppData;
use crate::common::json_error::{JsonError, ToJsonError};
use actix_web::error::ErrorNotFound;
use actix_web::web::Data;
use actix_web::{Error, HttpMessage, HttpRequest, HttpResponse};
use entity::students;
use serde::Serialize;
use utoipa::ToSchema;

/// Represents the response structure for retrieving a user's profile information.
///
/// This struct includes details about the student, such as their ID, name, email,
/// university affiliation, and role within the system. It is used in API responses to provide
/// comprehensive user profile data.
#[derive(Debug, Serialize, ToSchema)]
pub(crate) struct GetMeResponse {
    /// Student id
    #[schema(example = 253)]
    pub student_id: i32,
    /// First name of the student
    #[schema(example = "John")]
    pub first_name: String,
    /// Last name of the student
    #[schema(example = "Doe")]
    pub last_name: String,
    /// Email address associated with the student's account
    #[schema(format = "email", example = "john.doe@studenti.unitn.it")]
    pub email: String,
    /// Identifier for the university affiliated with the student
    #[schema(example = 123456)]
    pub university_id: i32,
}

#[utoipa::path(
    get,
    path = "/v1/users/me",
    responses(
        (status = 200, description = "Successfully retrieved user profile", body = GetMeResponse),
        (status = 404, description = "User not found in request context", body = JsonError),
        (status = 500, description = "Internal server error during serialization or database query", body = JsonError)
    ),
    tag = "Users",
)]
/// Returns authenticated user's profile information
///
/// Extracts user data from request extensions (set by auth middleware),
/// filters sensitive fields, and returns user data with associated projects and roles.
pub(super) async fn me_handler(
    req: HttpRequest, app_state: Data<AppData>,
) -> Result<HttpResponse, Error> {
    let user = match req.extensions().get::<students::Model>() {
        None => return Err(ErrorNotFound("user does not exists".to_json_error())),
        Some(u) => u.clone(),
    };

    let response: GetMeResponse = user.into();
    Ok(HttpResponse::Ok().json(response))
}

impl From<students::Model> for GetMeResponse {
    fn from(value: students::Model) -> Self {
        Self {
            student_id: value.student_id,
            first_name: value.first_name,
            last_name: value.last_name,
            email: value.email,
            university_id: value.university_id,
        }
    }
}
