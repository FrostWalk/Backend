use crate::database::repository_methods::RepositoryMethods;
use derive_new::new;
use entity::complaints::{ActiveModel, Entity};
use sea_orm::{DatabaseConnection, EntityTrait};

#[derive(new)]
pub struct ComplaintsRepository {
    db_conn: DatabaseConnection,
}

impl RepositoryMethods<Entity, ActiveModel> for ComplaintsRepository {
    fn db_conn(&self) -> &DatabaseConnection {
        &self.db_conn
    }
}
