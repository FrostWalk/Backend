use crate::api::v1::students::users::me::students_me_handler;
use crate::api::v1::students::users::update_me::update_me_student_handler;
use crate::jwt::student_auth_factory::Student;
use actix_web::{web, Scope};

pub(crate) mod me;
pub(crate) mod update_me;

pub(super) fn users_scope() -> Scope {
    web::scope("/users")
        .route(
            "/me",
            web::get()
                .to(students_me_handler)
                .wrap(Student::require_auth()),
        )
        .route(
            "/me",
            web::patch()
                .to(update_me_student_handler)
                .wrap(Student::require_auth()),
        )
}
