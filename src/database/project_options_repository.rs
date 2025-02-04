use crate::database::repository_methods::RepositoryMethods;
use derive_new::new;
use entity::project_options::ActiveModel;
use entity::project_options::Entity;
use repository_macro::RepositoryMethods;
use sea_orm::DatabaseConnection;

#[derive(new, RepositoryMethods, Clone)]
pub(crate) struct ProjectOptionsRepository {
    db_conn: DatabaseConnection,
}
