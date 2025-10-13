pub(crate) mod allowed_domains;
pub(crate) mod confirm;
pub(crate) mod forgot_password;
pub(crate) mod login;
pub(crate) mod reset_password;
pub(crate) mod signup;

use crate::api::v1::students::auth::{
    allowed_domains::allowed_domains_handler, confirm::confirm_student_handler,
    forgot_password::forgot_password_handler, login::students_login_handler,
    reset_password::reset_password_handler, signup::student_signup_handler,
};
use actix_web::{web, Scope};

pub(super) fn auth_scope() -> Scope {
    web::scope("/auth")
        .route("/login", web::post().to(students_login_handler))
        .route("/confirm", web::get().to(confirm_student_handler))
        .route("/signup", web::post().to(student_signup_handler))
        .route("/forgot-password", web::post().to(forgot_password_handler))
        .route("/reset-password", web::post().to(reset_password_handler))
        .route("/allowed-domains", web::get().to(allowed_domains_handler))
}
