pub(crate) mod login;

use crate::api::v1::auth::login::login_handler;
use actix_web::{web, Scope};

pub(super) fn auth_scope() -> Scope {
    web::scope("/auth").route("/login", web::post().to(login_handler))
}
