pub(crate) mod login;

use actix_web::{web, Scope};
use crate::api::v1::students::auth::login::students_login_handler;

pub(super) fn auth_scope() -> Scope {
    web::scope("/auth").route("/login", web::post().to(students_login_handler))
}
