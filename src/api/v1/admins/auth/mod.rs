use crate::api::v1::admins::auth::login::admins_login_handler;
use actix_web::{web, Scope};

pub(crate) mod login;

pub(super) fn auth_scope() -> Scope {
    web::scope("/auth").route("/login", web::post().to(admins_login_handler))
}
