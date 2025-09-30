use crate::api::v1::admins::auth::forgot_password::forgot_password_handler;
use crate::api::v1::admins::auth::login::admins_login_handler;
use crate::api::v1::admins::auth::reset_password::reset_password_handler;
use actix_web::{web, Scope};

pub(crate) mod forgot_password;
pub(crate) mod login;
pub(crate) mod reset_password;

pub(super) fn auth_scope() -> Scope {
    web::scope("/auth")
        .route("/login", web::post().to(admins_login_handler))
        .route("/forgot-password", web::post().to(forgot_password_handler))
        .route("/reset-password", web::post().to(reset_password_handler))
}
