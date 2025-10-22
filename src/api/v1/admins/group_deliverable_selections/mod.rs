use crate::api::v1::admins::group_deliverable_selections::read::get_group_deliverable_selections;
use actix_web::{web, Scope};

pub(crate) mod read;

pub(super) fn group_deliverable_selections_scope() -> Scope {
    web::scope("/group-deliverable-selections").route(
        "/projects/{project_id}",
        web::get().to(get_group_deliverable_selections),
    )
}
