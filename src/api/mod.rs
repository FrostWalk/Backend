use crate::api::v1::v1_scope;
use actix_web::web;
use doc::open_api;

pub(super) mod doc;
pub(super) mod v1;

pub(super) fn configure_endpoints(conf: &mut web::ServiceConfig) {
    conf.service(v1_scope()).service(open_api());
}
