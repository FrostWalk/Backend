use actix_web::dev::Extensions;
use entity::admins;
use entity::students;
use std::cell::Ref;

pub(crate) trait LoggedUser {
    fn get_admin(&self) -> Result<admins::Model, &'static str>;
    fn get_student(&self) -> Result<students::Model, &'static str>;
    const NOT_FOUND_ERROR: &'static str = "unable to extract user from extension";
}

impl LoggedUser for Ref<'_, Extensions> {
    fn get_admin(&self) -> Result<admins::Model, &'static str> {
        match self.get::<admins::Model>() {
            None => Err(Self::NOT_FOUND_ERROR),
            Some(u) => Ok(u.clone()),
        }
    }

    fn get_student(&self) -> Result<students::Model, &'static str> {
        match self.get::<students::Model>() {
            None => Err(Self::NOT_FOUND_ERROR),
            Some(u) => Ok(u.clone()),
        }
    }
}
