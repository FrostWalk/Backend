use crate::app_data::AppData;
use crate::common::json_error::{error_with_log_id_and_payload, JsonError, ToJsonError};
use crate::jwt::get_user::LoggedUser;
use crate::models::admin::Admin;
use crate::models::admin_role::AvailableAdminRole;
use actix_web::http::StatusCode;
use actix_web::web::{Data, Json};
use actix_web::{HttpMessage, HttpRequest, HttpResponse};
use log::{error, warn};
use password_auth::generate_hash;
use rand::Rng;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use welds::state::DbState;

#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub(crate) struct CreateAdminScheme {
    #[schema(example = "John")]
    pub first_name: String,
    #[schema(example = "Doe")]
    pub last_name: String,
    #[schema(example = "john.doe@example.com")]
    pub email: String,
    #[schema(example = "2")]
    pub admin_role_id: i32,
}

#[derive(Debug, Serialize, ToSchema)]
pub(crate) struct CreateAdminResponse {
    #[schema(example = "12345")]
    pub admin_id: i32,
}
#[utoipa::path(
    post,
    path = "/v1/admins/users",
    request_body = CreateAdminScheme,
    responses(
        (status = 200, description = "Admin created successfully", body = CreateAdminResponse),
        (status = 400, description = "Invalid data in request", body = JsonError),
        (status = 401, description = "Authentication required", body = JsonError),
        (status = 500, description = "Internal server error occurred", body = JsonError)
    ),
    security(("AdminAuth" = [])),
    tag = "Admin users management",
)]
/// Creates a new admin user.
///
/// This endpoint allows authenticated users to create new admin accounts. Only users with the root role can create other root users.
/// A random password is automatically generated and sent to the admin via email.
pub(super) async fn create_admin_handler(
    req: HttpRequest, payload: Json<CreateAdminScheme>, data: Data<AppData>,
) -> Result<HttpResponse, JsonError> {
    let user = match req.extensions().get_admin() {
        Ok(user) => user,
        Err(e) => {
            error!("entered a protected route without a user loaded in the request");
            return Err(e.to_json_error(StatusCode::INTERNAL_SERVER_ERROR));
        }
    };

    if (user.admin_role_id != AvailableAdminRole::Root as i32)
        && (payload.admin_role_id == AvailableAdminRole::Root as i32)
    {
        warn!("user {} tried to create a root user", user.email);
        return Err("Operation not permitted".to_json_error(StatusCode::FORBIDDEN));
    }

    // Generate a random secure password (16 characters, alphanumeric)
    let mut rng = rand::rng();
    const CHARS: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789";
    let generated_password: String = (0..16)
        .map(|_| {
            let idx = rng.random_range(0..CHARS.len());
            CHARS[idx] as char
        })
        .collect();

    let mut state = DbState::new_uncreated(Admin {
        admin_id: 0,
        first_name: payload.first_name.clone(),
        last_name: payload.last_name.clone(),
        email: payload.email.clone(),
        password_hash: generate_hash(&generated_password),
        admin_role_id: payload.admin_role_id,
    });

    if let Err(e) = state.save(&data.db).await {
        return Err(error_with_log_id_and_payload(
            format!("unable to create admin: {}", e),
            "Failed to create user",
            StatusCode::INTERNAL_SERVER_ERROR,
            log::Level::Error,
            &payload,
        ));
    }

    // Send welcome email with credentials
    let full_name = format!("{} {}", payload.first_name, payload.last_name);
    if let Err(e) = data
        .mailer
        .send_admin_welcome(payload.email.clone(), full_name, generated_password)
        .await
    {
        error!("Failed to send welcome email to {}: {}", payload.email, e);
        // Note: We continue even if email fails, as the admin was already created
        // The professor can manually share credentials if needed
    }

    Ok(HttpResponse::Ok().json(CreateAdminResponse {
        admin_id: state.admin_id,
    }))
}
