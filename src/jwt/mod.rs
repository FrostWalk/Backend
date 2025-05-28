use actix_web::dev::ServiceRequest;
use crate::app_state::AppState;

pub(super) mod admin_auth_factory;
mod admin_auth_middleware;
mod student_auth_middleware;
pub(crate) mod token;
