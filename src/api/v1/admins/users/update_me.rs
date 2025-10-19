use crate::app_data::AppData;
use crate::common::json_error::{
    error_with_log_id, error_with_log_id_and_payload, JsonError, ToJsonError,
};
use crate::database::repositories::admins_repository;
use crate::jwt::get_user::LoggedUser;
use actix_web::http::StatusCode;
use actix_web::web::{Data, Json};
use actix_web::{HttpMessage, HttpRequest, HttpResponse};
use password_auth::{generate_hash, verify_password};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub(crate) struct UpdateMeAdminScheme {
    #[schema(example = "OldPassword123")]
    pub old_password: String,
    #[schema(example = "John")]
    pub first_name: Option<String>,
    #[schema(example = "Doe")]
    pub last_name: Option<String>,
    #[schema(example = "john.doe@example.com")]
    pub email: Option<String>,
    #[schema(example = "NewSecureP@ss123")]
    pub password: Option<String>,
}

#[utoipa::path(
    patch,
    path = "/v1/admins/users/me",
    request_body = UpdateMeAdminScheme,
    responses(
        (status = 200, description = "Admin profile updated successfully"),
        (status = 400, description = "Invalid data in request", body = JsonError),
        (status = 401, description = "Authentication required", body = JsonError),
        (status = 409, description = "Email already exists", body = JsonError),
        (status = 500, description = "Internal server error occurred", body = JsonError)
    ),
    security(("AdminAuth" = [])),
    tag = "Admin users management",
)]
/// Updates the currently authenticated admin's profile.
///
/// This endpoint allows admins to update their own profile details including name, email, and password.
pub(super) async fn update_me_admin_handler(
    req: HttpRequest, 
    body: Json<UpdateMeAdminScheme>, 
    data: Data<AppData>,
) -> Result<HttpResponse, JsonError> {
    let user = match req.extensions().get_admin() {
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

    // Load the current admin from database to verify password
    let admin_state_opt = admins_repository::get_by_id(&data.db, user.admin_id)
        .await
        .map_err(|e| {
            error_with_log_id_and_payload(
                format!("unable to load admin {}: {}", user.admin_id, e),
                "Profile update failed",
                StatusCode::INTERNAL_SERVER_ERROR,
                log::Level::Error,
                &body,
            )
        })?;

    let mut admin_state = match admin_state_opt {
        Some(s) => s,
        None => return Err("Admin not found".to_json_error(StatusCode::NOT_FOUND)),
    };

    // Verify old password
    if verify_password(&body.old_password, &admin_state.password_hash).is_err() {
        return Err("Incorrect password".to_json_error(StatusCode::UNAUTHORIZED));
    }

    // Validate that at least one field is being updated
    if body.first_name.is_none()
        && body.last_name.is_none()
        && body.email.is_none()
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
            let email_exists = admins_repository::get_by_email(&data.db, new_email)
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

    // Apply only provided fields
    if let Some(v) = body.first_name.clone() {
        admin_state.first_name = v;
    }
    if let Some(v) = body.last_name.clone() {
        admin_state.last_name = v;
    }
    if let Some(v) = body.email.clone() {
        admin_state.email = v;
    }
    if let Some(v) = body.password.clone() {
        admin_state.password_hash = generate_hash(v);
    }

    admin_state.save(&data.db).await.map_err(|e| {
        error_with_log_id_and_payload(
            format!("unable to update admin profile: {}", e),
            "Profile update failed",
            StatusCode::INTERNAL_SERVER_ERROR,
            log::Level::Error,
            &body,
        )
    })?;

    Ok(HttpResponse::Ok().finish())
}
