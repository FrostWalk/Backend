use crate::database::repository_methods::RepositoryMethods;
use derive_new::new;
use entity::roles::{ActiveModel, Entity};
use sea_orm::DatabaseConnection;

#[derive(new)]
pub(crate) struct RolesRepository {
    db_conn: DatabaseConnection,
}

impl  RepositoryMethods<Entity, ActiveModel> for RolesRepository{
    fn db_conn(&self) -> &DatabaseConnection {
        &self.db_conn
    }
}