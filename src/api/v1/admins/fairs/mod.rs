use crate::api::v1::admins::fairs::create::create_fair_handler;
use crate::api::v1::admins::fairs::disable::disable_fair_handler;
use crate::api::v1::admins::fairs::enable::enable_fair_handler;
use crate::api::v1::admins::fairs::read::{get_fair_by_project_handler, get_fair_handler};
use crate::api::v1::admins::fairs::report::fair_report_handler;
use crate::api::v1::admins::fairs::update::update_fair_handler;
use actix_web::{web, Scope};

pub(crate) mod create;
pub(crate) mod disable;
pub(crate) mod enable;
pub(crate) mod read;
pub(crate) mod report;
pub(crate) mod update;

pub(super) fn fairs_scope() -> Scope {
    web::scope("/fairs")
        .route("", web::post().to(create_fair_handler))
        .route("/{fair_id}", web::get().to(get_fair_handler))
        .route("/{fair_id}", web::patch().to(update_fair_handler))
        .route("/{fair_id}/enable", web::post().to(enable_fair_handler))
        .route("/{fair_id}/disable", web::post().to(disable_fair_handler))
        .route("/{fair_id}/report", web::get().to(fair_report_handler))
        .route(
            "/project/{project_id}",
            web::get().to(get_fair_by_project_handler),
        )
}
