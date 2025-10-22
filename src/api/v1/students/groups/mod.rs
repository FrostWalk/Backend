use crate::api::v1::students::groups::check_name::check_name;
use crate::api::v1::students::groups::create::create_group;
use crate::api::v1::students::groups::delete::delete_group;
use crate::api::v1::students::groups::members::{add_member, remove_member};
use crate::api::v1::students::groups::members_list::list_group_members;
use crate::api::v1::students::groups::read::get_groups;
use actix_web::{web, Scope};

pub(crate) mod check_name;
pub(crate) mod create;
pub(crate) mod delete;
pub(crate) mod members;
pub(crate) mod members_list;
pub(crate) mod read;

pub(super) fn groups_scope() -> Scope {
    web::scope("/groups")
        .route("", web::post().to(create_group))
        .route("", web::get().to(get_groups))
        .route("/check-name", web::post().to(check_name))
        .route("/{group_id}", web::delete().to(delete_group))
        .route("/{group_id}/members", web::get().to(list_group_members))
        .route("/{group_id}/members", web::post().to(add_member))
        .route("/{group_id}/members", web::delete().to(remove_member))
}
