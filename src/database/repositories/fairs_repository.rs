use crate::database::repository_methods_trait::RepositoryMethods;
use derive_new::new;
use entity::fairs::ActiveModel;
use entity::fairs::Entity;
use repository_macro::RepositoryMethods;
use sea_orm::DatabaseConnection;

#[derive(new, RepositoryMethods, Clone)]
pub struct FairsRepository {
    db_conn: DatabaseConnection,
}
