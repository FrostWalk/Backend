use crate::database::repository_methods::RepositoryMethods;
use derive_new::new;
use entity::fair::ActiveModel;
use entity::fair::Entity;
use repository_macro::RepositoryMethods;
use sea_orm::DatabaseConnection;

#[derive(new, RepositoryMethods, Clone)]
pub struct FairRepository {
    db_conn: DatabaseConnection,
}
