use crate::common::json_error::{error_with_log_id, JsonError};
use crate::jwt::get_user::LoggedUser;
use crate::models::student::Student;
use actix_web::http::StatusCode;
use actix_web::{HttpMessage, HttpRequest, HttpResponse};
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
    security(("StudentAuth" = [])),
    tag = "Student users management",
)]
/// Retrieves the profile information of the currently authenticated student.
///
/// This endpoint is designed to return detailed information about the student making the request.
/// It extracts the student's data from the request context, which should be populated by middleware
/// responsible for authentication and authorization.
#[actix_web_grants::protect("ROLE_STUDENT")]
pub(super) async fn students_me_handler(req: HttpRequest) -> Result<HttpResponse, JsonError> {
    let user = match req.extensions().get_student() {
        Ok(user) => user,
        Err(_) => {
            return Err(error_with_log_id(
                "entered a protected route without a user loaded in the request",
                "Authentication error",
                StatusCode::INTERNAL_SERVER_ERROR,
                log::Level::Error,
            ));
        }
    };

    let response: GetMeStudentResponse = user.into();
    Ok(HttpResponse::Ok().json(response))
}

impl From<Student> for GetMeStudentResponse {
    fn from(value: Student) -> Self {
        Self {
            id: value.student_id,
            first_name: value.first_name,
            last_name: value.last_name,
            email: value.email,
            university_id: value.university_id,
        }
    }
}
