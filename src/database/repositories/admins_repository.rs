use crate::database::repository_methods_trait::RepositoryMethods;
use derive_new::new;
use entity::admins::ActiveModel;
use entity::admins::Entity;
use repository_macro::RepositoryMethods;
use sea_orm::DatabaseConnection;

#[derive(new, RepositoryMethods, Clone)]
pub(crate) struct AdminsRepository {
    db_conn: DatabaseConnection,
}

impl AdminsRepository {}
