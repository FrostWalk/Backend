use crate::api::v1::admins::blacklist::create::add_to_blacklist_handler;
use crate::api::v1::admins::blacklist::delete::delete_blacklist_handler;
use crate::api::v1::admins::blacklist::get::get_blacklist_handler;
use crate::api::v1::admins::blacklist::list::list_blacklist_handler;
use crate::api::v1::admins::blacklist::update::update_blacklist_handler;
use actix_web::{web, Scope};

pub(crate) mod create;
pub(crate) mod delete;
pub(crate) mod get;
pub(crate) mod list;
pub(crate) mod update;

pub(super) fn blacklist_scope() -> Scope {
    web::scope("/blacklist")
        .route("", web::post().to(add_to_blacklist_handler))
        .route("", web::get().to(list_blacklist_handler))
        .route("/{blacklist_id}", web::get().to(get_blacklist_handler))
        .route("/{blacklist_id}", web::patch().to(update_blacklist_handler))
        .route(
            "/{blacklist_id}",
            web::delete().to(delete_blacklist_handler),
        )
}
