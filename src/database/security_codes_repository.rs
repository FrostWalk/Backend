use crate::database::repository_methods::RepositoryMethods;
use derive_new::new;
use entity::security_codes::{ActiveModel, Entity};
use sea_orm::DatabaseConnection;
use repository_macro::RepositoryMethods;

#[derive(new, RepositoryMethods)]
pub(crate) struct SecurityCodesRepository {
    db_conn: DatabaseConnection,
}