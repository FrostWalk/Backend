use crate::database::repository_methods::RepositoryMethods;
use derive_new::new;
use entity::projects::{ActiveModel, Entity};
use sea_orm::DatabaseConnection;

#[derive(new)]
pub(crate) struct ProjectRepository {
    db_conn: DatabaseConnection,
}

impl RepositoryMethods<Entity, ActiveModel> for ProjectRepository {
    fn db_conn(&self) -> &DatabaseConnection {
        &self.db_conn
    }
}