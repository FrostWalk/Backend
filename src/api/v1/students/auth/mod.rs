pub(crate) mod login;

use crate::api::v1::students::auth::login::students_login_handler;
use actix_web::{web, Scope};

pub(super) fn auth_scope() -> Scope {
    web::scope("/auth").route("/login", web::post().to(students_login_handler))
}
