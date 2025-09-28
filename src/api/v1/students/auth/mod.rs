pub(crate) mod confirm;
pub(crate) mod login;
pub(crate) mod signup;

use crate::api::v1::students::auth::{
    confirm::confirm_student_handler, login::students_login_handler, signup::student_signup_handler,
};
use actix_web::{web, Scope};

pub(super) fn auth_scope() -> Scope {
    web::scope("/auth")
        .route("/login", web::post().to(students_login_handler))
        .route("/confirm", web::get().to(confirm_student_handler))
        .route("/signup", web::post().to(student_signup_handler))
}
