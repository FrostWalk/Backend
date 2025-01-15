use crate::database::repository_methods::RepositoryMethods;
use derive_new::new;
use entity::groups::{ActiveModel, Entity};
use sea_orm::DatabaseConnection;

#[derive(new)]
pub(crate) struct GroupsRepository {
    db_conn: DatabaseConnection,
}

impl RepositoryMethods<Entity, ActiveModel> for GroupsRepository{
    fn db_conn(&self) -> &DatabaseConnection {
        &self.db_conn
    }
}