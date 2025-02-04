use crate::database::repository_methods::RepositoryMethods;
use derive_new::new;
use entity::auxiliary_roles::ActiveModel;
use entity::auxiliary_roles::Entity;
use repository_macro::RepositoryMethods;
use sea_orm::DatabaseConnection;

#[derive(new, RepositoryMethods, Clone)]
pub struct AuxiliaryRolesRepository {
    db_conn: DatabaseConnection,
}
