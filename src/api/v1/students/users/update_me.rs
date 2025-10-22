use crate::app_data::AppData;
use crate::common::json_error::{
    error_with_log_id, error_with_log_id_and_payload, JsonError, ToJsonError,
};
use crate::database::repositories::students_repository;
use crate::jwt::get_user::LoggedUser;
use actix_web::http::StatusCode;
use actix_web::web::{Data, Json};
use actix_web::{HttpMessage, HttpRequest, HttpResponse};
use password_auth::{generate_hash, verify_password};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub(crate) struct UpdateMeStudentScheme {
    #[schema(example = "OldPassword123")]
    pub old_password: String,
    #[schema(example = "John")]
    pub first_name: Option<String>,
    #[schema(example = "Doe")]
    pub last_name: Option<String>,
    #[schema(example = "john.doe@studenti.unitn.it")]
    pub email: Option<String>,
    #[schema(example = 123456)]
    pub university_id: Option<i32>,
    #[schema(example = "NewSecureP@ss123")]
    pub password: Option<String>,
}

#[utoipa::path(
    patch,
    path = "/v1/students/users/me",
    request_body = UpdateMeStudentScheme,
    responses(
        (status = 200, description = "Student profile updated successfully"),
        (status = 400, description = "Invalid data in request", body = JsonError),
        (status = 401, description = "Authentication required", body = JsonError),
        (status = 409, description = "Email already exists", body = JsonError),
        (status = 500, description = "Internal server error occurred", body = JsonError)
    ),
    security(("UserAuth" = [])),
    tag = "Student users management",
)]
/// Updates the currently authenticated student's profile.
///
/// This endpoint allows students to update their own profile details including name, email, and password.
#[actix_web_grants::protect("ROLE_STUDENT")]
pub(super) async fn update_me_student_handler(
    req: HttpRequest, body: Json<UpdateMeStudentScheme>, data: Data<AppData>,
) -> Result<HttpResponse, JsonError> {
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

    // Validate old password is not empty
    if body.old_password.trim().is_empty() {
        return Err("Old password is required".to_json_error(StatusCode::BAD_REQUEST));
    }

    // Load the current student from database to verify password
    let student_state_opt = students_repository::get_by_id(&data.db, user.student_id)
        .await
        .map_err(|e| {
            error_with_log_id_and_payload(
                format!("unable to load student {}: {}", user.student_id, e),
                "Profile update failed",
                StatusCode::INTERNAL_SERVER_ERROR,
                log::Level::Error,
                &body,
            )
        })?;

    let mut student_state = match student_state_opt {
        Some(s) => s,
        None => return Err("Student not found".to_json_error(StatusCode::NOT_FOUND)),
    };

    // Verify old password
    if verify_password(&body.old_password, &student_state.password_hash).is_err() {
        return Err("Incorrect password".to_json_error(StatusCode::UNAUTHORIZED));
    }

    // Validate that at least one field is being updated
    if body.first_name.is_none()
        && body.last_name.is_none()
        && body.email.is_none()
        && body.university_id.is_none()
        && body.password.is_none()
    {
        return Err("At least one field must be provided".to_json_error(StatusCode::BAD_REQUEST));
    }

    // Validate that fields are not empty strings
    if let Some(ref first_name) = body.first_name {
        if first_name.trim().is_empty() {
            return Err("First name cannot be empty".to_json_error(StatusCode::BAD_REQUEST));
        }
    }
    if let Some(ref last_name) = body.last_name {
        if last_name.trim().is_empty() {
            return Err("Last name cannot be empty".to_json_error(StatusCode::BAD_REQUEST));
        }
    }
    if let Some(ref email) = body.email {
        if email.trim().is_empty() {
            return Err("Email cannot be empty".to_json_error(StatusCode::BAD_REQUEST));
        }
    }
    if let Some(ref password) = body.password {
        if password.trim().is_empty() {
            return Err("Password cannot be empty".to_json_error(StatusCode::BAD_REQUEST));
        }
    }

    // If email is being changed, check if it already exists (for another user)
    if let Some(ref new_email) = body.email {
        if new_email != &user.email {
            let email_exists = students_repository::get_by_email(&data.db, new_email)
                .await
                .map_err(|e| {
                    error_with_log_id_and_payload(
                        format!("unable to check if email exists: {}", e),
                        "Profile update failed",
                        StatusCode::INTERNAL_SERVER_ERROR,
                        log::Level::Error,
                        &body,
                    )
                })?;

            if email_exists.is_some() {
                return Err(
                    "Email already in use by another account".to_json_error(StatusCode::CONFLICT)
                );
            }
        }
    }

    // If university_id is being changed, check if it already exists (for another user)
    if let Some(new_university_id) = body.university_id {
        if new_university_id != user.university_id {
            let university_id_exists =
                students_repository::university_id_exists(&data.db, new_university_id)
                    .await
                    .map_err(|e| {
                        error_with_log_id_and_payload(
                            format!("unable to check if university ID exists: {}", e),
                            "Profile update failed",
                            StatusCode::INTERNAL_SERVER_ERROR,
                            log::Level::Error,
                            &body,
                        )
                    })?;

            if university_id_exists {
                return Err("University ID already in use by another account"
                    .to_json_error(StatusCode::CONFLICT));
            }
        }
    }

    // Apply only provided fields
    if let Some(v) = body.first_name.clone() {
        student_state.first_name = v;
    }
    if let Some(v) = body.last_name.clone() {
        student_state.last_name = v;
    }
    if let Some(v) = body.email.clone() {
        student_state.email = v;
    }
    if let Some(v) = body.university_id {
        student_state.university_id = v;
    }
    if let Some(v) = body.password.clone() {
        student_state.password_hash = generate_hash(v);
    }

    students_repository::update(&data.db, student_state)
        .await
        .map_err(|e| {
            error_with_log_id_and_payload(
                format!("unable to update student profile: {}", e),
                "Profile update failed",
                StatusCode::INTERNAL_SERVER_ERROR,
                log::Level::Error,
                &body,
            )
        })?;

    Ok(HttpResponse::Ok().finish())
}
