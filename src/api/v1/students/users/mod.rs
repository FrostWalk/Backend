use crate::api::v1::students::users::me::students_me_handler;
use crate::api::v1::students::users::signup::student_signup_handler;
use crate::jwt::student_auth_factory::Student;
use actix_web::{web, Scope};

pub(crate) mod me;
pub(crate) mod signup;

pub(super) fn users_scope() -> Scope {
    web::scope("/users")
        .route("/signup", web::post().to(student_signup_handler))
        .route(
            "/me",
            web::get()
                .to(students_me_handler)
                .wrap(Student::require_auth()),
        )
}
