use derive_new::new;
use sea_orm::DatabaseConnection;

#[derive(new)]
pub(crate) struct UsersProjectsRolesRepository {
    db_conn: DatabaseConnection,
}
