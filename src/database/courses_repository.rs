use crate::database::repository_methods::RepositoryMethods;
use derive_new::new;
use entity::courses::{ActiveModel, Entity};
use sea_orm::{DatabaseConnection, EntityTrait};

#[derive(new)]
pub(crate) struct CoursesRepository {
    db_conn: DatabaseConnection,
}

impl RepositoryMethods<Entity, ActiveModel> for CoursesRepository {
    fn db_conn(&self) -> &DatabaseConnection {
        &self.db_conn
    }
}
