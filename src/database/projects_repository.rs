use crate::database::repository_methods::RepositoryMethods;
use derive_new::new;
use entity::projects::{ActiveModel, Entity};
use sea_orm::DatabaseConnection;
use repository_macro::RepositoryMethods;

#[derive(new, RepositoryMethods)]
pub(crate) struct ProjectRepository {
    db_conn: DatabaseConnection,
}
