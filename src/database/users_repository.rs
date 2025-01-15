use crate::database::repository_methods::RepositoryMethods;
use derive_new::new;
use entity::users::{ActiveModel, Entity};
use sea_orm::DatabaseConnection;

#[derive(new)]
pub(crate) struct UsersRepository {
    db_conn: DatabaseConnection,
}

impl RepositoryMethods<Entity, ActiveModel> for UsersRepository {
    fn db_conn(&self) -> &DatabaseConnection {
        &self.db_conn
    }
}
