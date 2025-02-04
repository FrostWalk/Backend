pub(crate) mod login;
pub(crate) mod logout;

use crate::api::v1::auth::login::login_handler;
use crate::api::v1::auth::logout::logout_handler;
use actix_web::{web, Scope};

pub(super) fn auth_scope() -> Scope {
    web::scope("/auth")
        .route("/login", web::post().to(login_handler))
        .route("/logout", web::post().to(logout_handler))
}
