use crate::database::repository_methods::RepositoryMethods;
use derive_new::new;
use entity::security_codes::{ActiveModel, Entity};
use sea_orm::DatabaseConnection;

#[derive(new)]
pub(crate) struct SecurityCodesRepository {
    db_conn: DatabaseConnection,
}

impl RepositoryMethods<Entity, ActiveModel> for SecurityCodesRepository {
    fn db_conn(&self) -> &DatabaseConnection {
        &self.db_conn
    }
}
