use crate::api::v1::students::group_deliverable_selections::create::create_group_deliverable_selection;
use crate::api::v1::students::group_deliverable_selections::read::get_group_deliverable_selection;
use crate::api::v1::students::group_deliverable_selections::update::update_group_deliverable_selection;
use crate::jwt::student_auth_factory::Student;
use actix_web::{web, Scope};

pub(crate) mod create;
pub(crate) mod read;
pub(crate) mod update;

pub(super) fn group_deliverable_selections_scope() -> Scope {
    web::scope("/group-deliverable-selections")
        .route(
            "/{group_id}",
            web::post()
                .to(create_group_deliverable_selection)
                .wrap(Student::require_auth()),
        )
        .route(
            "/{group_id}",
            web::get()
                .to(get_group_deliverable_selection)
                .wrap(Student::require_auth()),
        )
        .route(
            "/{group_id}",
            web::patch()
                .to(update_group_deliverable_selection)
                .wrap(Student::require_auth()),
        )
}
