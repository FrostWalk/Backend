use crate::common::json_error::ToJsonError;
use actix_web::dev::Extensions;
use actix_web::error::ErrorInternalServerError;
use entity::admins;
use entity::students;
use std::cell::Ref;
use std::error::Error;

pub(crate) trait LoggedUser {
    fn get_admin(&self) -> Result<admins::Model, Box<dyn Error>>;
    fn get_student(&self) -> Result<students::Model, Box<dyn Error>>;
    const NOT_FOUND_ERROR: &'static str = "unable to extract user from extension";
}

impl LoggedUser for Ref<'_, Extensions> {
    fn get_admin(&self) -> Result<admins::Model, Box<dyn Error>> {
        match self.get::<admins::Model>() {
            None => Err(Box::new(ErrorInternalServerError(
                Self::NOT_FOUND_ERROR.to_json_error(),
            ))),
            Some(u) => Ok(u.clone()),
        }
    }

    fn get_student(&self) -> Result<students::Model, Box<dyn Error>> {
        match self.get::<students::Model>() {
            None => Err(Box::new(ErrorInternalServerError(
                Self::NOT_FOUND_ERROR.to_json_error(),
            ))),
            Some(u) => Ok(u.clone()),
        }
    }
}
