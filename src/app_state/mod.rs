use derive_new::new;
use sea_orm::DatabaseConnection;

#[derive(new)]
pub(crate) struct AppState {
    pub(crate) db_connection: DatabaseConnection,
}
