use crate::api::v1::students::projects::read::get_student_projects;
use actix_web::{web, Scope};

pub(crate) mod read;

pub(super) fn projects_scope() -> Scope {
    web::scope("/projects").route("", web::get().to(get_student_projects))
}
