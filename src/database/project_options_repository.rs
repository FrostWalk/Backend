use crate::database::repository_methods::RepositoryMethods;
use derive_new::new;
use entity::project_options::{ActiveModel, Entity};
use sea_orm::{ActiveModelTrait, DatabaseConnection, EntityTrait};

#[derive(new)]
pub(crate) struct ProjectOptionsRepository {
    db_conn: DatabaseConnection,
}

impl RepositoryMethods<Entity, ActiveModel> for ProjectOptionsRepository {
    fn db_conn(&self) -> &DatabaseConnection {
        &self.db_conn
    }
}
