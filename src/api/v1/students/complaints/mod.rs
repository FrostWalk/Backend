use crate::api::v1::students::complaints::list::list_group_filed_complaints_handler;
use crate::api::v1::students::complaints::submit::submit_complaint_handler;
use actix_web::{web, Scope};

pub(crate) mod list;
pub(crate) mod submit;

pub(super) fn complaints_scope() -> Scope {
    web::scope("")
        .route("/complaints", web::post().to(submit_complaint_handler))
        .route(
            "/groups/{group_id}/complaints",
            web::get().to(list_group_filed_complaints_handler),
        )
}
