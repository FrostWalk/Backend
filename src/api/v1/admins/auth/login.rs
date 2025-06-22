use crate::app_data::AppData;
use crate::common::json_error::{database_error, JsonError, ToJsonError};
use crate::jwt::token::create_admin_token;
use actix_web::cookie::time::Duration;
use actix_web::http::StatusCode;
use actix_web::web::Data;
use actix_web::web::Json;
use actix_web::HttpResponse;
use log::error;
use password_auth::verify_password;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

const WRONG_CREDENTIALS: &str = "Incorrect email or password";

/// Represents data needed for login
#[derive(Deserialize, ToSchema)]
pub(crate) struct LoginAdminsSchema {
    #[schema(example = "user@example.com")]
    email: String,
    #[schema(example = "password123")]
    password: String,
}
/// Represents the response structure for a successful login.
///
/// This struct includes a JWT token that can be used for later authenticated requests.
#[derive(Serialize, ToSchema)]
pub(crate) struct LoginAdminsResponse {
    /// JSON Web Token (JWT) to be used for authentication in later requests.
    #[schema(example = "eyJhbGc9...")]
    token: String,
}

/// Authenticates an admin and returns a JWT.
///
/// This endpoint validates user credentials and issues a JWT upon successful authentication.
#[utoipa::path(
    post,
    path = "/v1/admins/auth/login",
    request_body = LoginAdminsSchema,
    responses(
        (status = 200, description = "Login successful", body = LoginAdminsResponse),
        (status = 401, description = "Wrong credentials", body = JsonError),
        (status = 500, description = "Internal server error", body = JsonError)
    ),
    tag = "Admin authentication"
)]
pub(crate) async fn admins_login_handler(
    req: Json<LoginAdminsSchema>, data: Data<AppData>,
) -> Result<HttpResponse, JsonError> {
    // convenience variable storing error in case of wrong credentials or user not found
    let unauthorized = Err(WRONG_CREDENTIALS.to_json_error(StatusCode::UNAUTHORIZED));

    // find the user in the db
    let opt = match data.repositories.admins.get_from_mail(&req.email).await {
        Ok(o) => o,
        Err(e) => {
            error!("unable to fetch admin from database: {}", e);
            return Err(database_error());
        }
    };

    // user is not found
    if opt.is_none() {
        return unauthorized;
    }

    let user = opt.unwrap();

    // password is incorrect
    if verify_password(&req.password, &user.password_hash).is_err() {
        return unauthorized;
    }

    // create jwt from user data if the creation fails return error 500
    let token = match create_admin_token(
        user.admin_id,
        user.admin_role_id,
        data.config.jwt_secret().as_bytes(),
        Duration::days(data.config.jwt_validity_days()).whole_seconds(),
    ) {
        Ok(t) => t,
        Err(e) => {
            error!("unable to create admin jwt token: {}", e);
            return Err(
                "unable to create jwt token".to_json_error(StatusCode::INTERNAL_SERVER_ERROR)
            );
        }
    };

    // return status code 200 with cookie
    Ok(HttpResponse::Ok().json(LoginAdminsResponse { token }))
}
