use crate::database::repository_methods_trait::RepositoryMethods;
use derive_new::new;
use entity::blacklist::ActiveModel;
use entity::blacklist::Entity;
use repository_macro::RepositoryMethods;
use sea_orm::DatabaseConnection;

#[derive(new, RepositoryMethods, Clone)]
pub struct BlacklistRepository {
    db_conn: DatabaseConnection,
}
