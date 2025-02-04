use crate::api::v1::doc::open_api;
use crate::api::v1::v1_scope;
use actix_web::web;

mod v1;

pub(super) fn configure_endpoints(conf: &mut web::ServiceConfig) {
    conf.service(v1_scope()).service(open_api());
}
