use entity::complaints::ActiveModel;
use entity::complaints::Entity;
use crate::database::repository_methods::RepositoryMethods;
use derive_new::new;
use sea_orm::{DatabaseConnection};
use repository_macro::RepositoryMethods;

#[derive(new, RepositoryMethods)]
pub struct ComplaintsRepository {
    db_conn: DatabaseConnection,
}
