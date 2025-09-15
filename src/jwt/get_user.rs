use crate::models::admin::Admin;
use crate::models::student::Student;
use actix_web::dev::Extensions;
use std::cell::Ref;

pub(crate) trait LoggedUser {
    fn get_admin(&self) -> Result<Admin, &'static str>;
    fn get_student(&self) -> Result<Student, &'static str>;
    const NOT_FOUND_ERROR: &'static str = "unable to extract user from extension";
}

impl LoggedUser for Ref<'_, Extensions> {
    fn get_admin(&self) -> Result<Admin, &'static str> {
        match self.get::<Admin>() {
            None => Err(Self::NOT_FOUND_ERROR),
            Some(u) => Ok(u.clone()),
        }
    }

    fn get_student(&self) -> Result<Student, &'static str> {
        match self.get::<Student>() {
            None => Err(Self::NOT_FOUND_ERROR),
            Some(u) => Ok(u.clone()),
        }
    }
}
