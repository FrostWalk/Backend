use crate::database::repository_methods_trait::RepositoryMethods;
use derive_new::new;
use entity::groups::{ActiveModel, Entity};
use repository_macro::RepositoryMethods;
use sea_orm::DatabaseConnection;

#[derive(new, RepositoryMethods, Clone)]
pub(crate) struct GroupsRepository {
    db_conn: DatabaseConnection,
}
