use crate::database::repository_methods::RepositoryMethods;
use derive_new::new;
use entity::courses::{ActiveModel, Entity};
use sea_orm::{DatabaseConnection};
use repository_macro::RepositoryMethods;

#[derive(new, RepositoryMethods)]
pub(crate) struct CoursesRepository {
    db_conn: DatabaseConnection,
}

