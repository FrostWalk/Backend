use crate::common::json_error::{JsonError, ToJsonError};
use actix_web::error::ErrorNotFound;
use actix_web::{Error, HttpMessage, HttpRequest, HttpResponse};
use entity::students;
use serde::Serialize;
use utoipa::ToSchema;

#[derive(Debug, Serialize, ToSchema)]
pub(crate) struct GetMeStudentResponse {
    #[schema(example = 253)]
    pub id: i32,
    #[schema(example = "John")]
    pub first_name: String,
    #[schema(example = "Doe")]
    pub last_name: String,
    #[schema(format = "email", example = "john.doe@studenti.unitn.it")]
    pub email: String,
    #[schema(example = 123456)]
    pub university_id: i32,
}

#[utoipa::path(
    get,
    path = "/v1/students/users/me",
    responses(
        (status = 200, description = "Successfully retrieved user profile", body = GetMeStudentResponse),
        (status = 404, description = "User not found in request context", body = JsonError),
        (status = 500, description = "Internal server error during serialization or database query", body = JsonError)
    ),
    tag = "Users",
)]
/// Retrieves the profile information of the currently authenticated student.
///
/// This endpoint is designed to return detailed information about the student making the request.
/// It extracts the student's data from the request context, which should be populated by middleware
/// responsible for authentication and authorization.
pub(super) async fn students_me_handler(req: HttpRequest) -> Result<HttpResponse, Error> {
    let user = match req.extensions().get::<students::Model>() {
        None => return Err(ErrorNotFound("user does not exists".to_json_error())),
        Some(u) => u.clone(),
    };

    let response: GetMeStudentResponse = user.into();
    Ok(HttpResponse::Ok().json(response))
}

impl From<students::Model> for GetMeStudentResponse {
    fn from(value: students::Model) -> Self {
        Self {
            id: value.student_id,
            first_name: value.first_name,
            last_name: value.last_name,
            email: value.email,
            university_id: value.university_id,
        }
    }
}
