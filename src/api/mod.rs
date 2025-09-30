use crate::api::health::{health_check, liveness_check};
use crate::api::v1::v1_scope;
use crate::api::version::version_info;
use actix_web::web;
use doc::open_api;

pub(super) mod doc;
pub(super) mod health;
pub(super) mod v1;
pub(super) mod version;

pub(super) fn configure_endpoints(conf: &mut web::ServiceConfig) {
    conf.service(v1_scope())
        .service(open_api())
        .route("/health", web::get().to(health_check))
        .route("/health/live", web::get().to(liveness_check))
        .route("/version", web::get().to(version_info));
}
