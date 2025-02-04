use crate::database::repository_methods::RepositoryMethods;
use derive_new::new;
use entity::security_codes::{ActiveModel, Entity};
use repository_macro::RepositoryMethods;
use sea_orm::DatabaseConnection;

#[derive(new, RepositoryMethods, Clone)]
pub(crate) struct SecurityCodesRepository {
    db_conn: DatabaseConnection,
}
