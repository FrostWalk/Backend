use crate::database::repository_methods::RepositoryMethods;
use derive_new::new;
use entity::complaints::ActiveModel;
use entity::complaints::Entity;
use repository_macro::RepositoryMethods;
use sea_orm::DatabaseConnection;

#[derive(new, RepositoryMethods)]
pub struct ComplaintsRepository {
    db_conn: DatabaseConnection,
}
