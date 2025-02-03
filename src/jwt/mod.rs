pub(super) mod auth_factory;
mod auth_middleware;
pub(super) mod role;
pub(crate) mod token;

pub(crate) const COOKIE_NAME: &str = "access_token";
