use crate::database::repository_methods_trait::RepositoryMethods;
use derive_new::new;
use entity::projects::{ActiveModel, Entity};
use repository_macro::RepositoryMethods;
use sea_orm::DatabaseConnection;

#[derive(new, RepositoryMethods, Clone)]
pub(crate) struct ProjectRepository {
    db_conn: DatabaseConnection,
}


impl ProjectRepository {
}
