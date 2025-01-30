use crate::database::repository_methods::RepositoryMethods;
use derive_new::new;
use entity::black_list::ActiveModel;
use entity::black_list::Entity;
use repository_macro::RepositoryMethods;
use sea_orm::DatabaseConnection;

#[derive(new, RepositoryMethods)]
pub struct BlacklistRepository {
    db_conn: DatabaseConnection,
}
