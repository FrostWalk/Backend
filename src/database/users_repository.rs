use crate::database::repository_methods::RepositoryMethods;
use derive_new::new;
use entity::users::ActiveModel;
use entity::users::Entity;
use repository_macro::RepositoryMethods;
use sea_orm::DatabaseConnection;

#[derive(new, RepositoryMethods)]
pub(crate) struct UsersRepository {
    db_conn: DatabaseConnection,
}
