use crate::database::repository_methods::RepositoryMethods;
use derive_new::new;
use entity::roles::{ActiveModel, Entity};
use repository_macro::RepositoryMethods;
use sea_orm::DatabaseConnection;

#[derive(new, RepositoryMethods, Clone)]
pub(crate) struct RolesRepository {
    db_conn: DatabaseConnection,
}
