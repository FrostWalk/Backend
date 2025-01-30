use crate::database::repository_methods::RepositoryMethods;
use derive_new::new;
use entity::groups::{ActiveModel, Entity};
use repository_macro::RepositoryMethods;
use sea_orm::DatabaseConnection;

#[derive(new, RepositoryMethods)]
pub(crate) struct GroupsRepository {
    db_conn: DatabaseConnection,
}
