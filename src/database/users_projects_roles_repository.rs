use derive_new::new;
use sea_orm::DatabaseConnection;

#[derive(new, Clone)]
pub(crate) struct UsersProjectsRolesRepository {
    db_conn: DatabaseConnection,
}
