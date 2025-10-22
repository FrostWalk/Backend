use crate::api::v1::admins::groups::details::get_group_details;
use crate::api::v1::admins::groups::members::{add_member, remove_member, transfer_leadership};
use crate::api::v1::admins::groups::read::get_project_groups;
use actix_web::{web, Scope};

pub(crate) mod details;
pub(crate) mod members;
pub(crate) mod read;

pub(super) fn groups_scope() -> Scope {
    web::scope("/groups")
        .route("/projects/{project_id}", web::get().to(get_project_groups))
        .route("/{group_id}", web::get().to(get_group_details))
        .route(
            "/{group_id}/members/{student_id}",
            web::delete().to(remove_member),
        )
        .route("/{group_id}/leader", web::patch().to(transfer_leadership))
        .route("/{group_id}/members", web::post().to(add_member))
}
