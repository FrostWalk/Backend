use crate::database::complaints_repository::ComplaintsRepository;
use crate::database::project_options_repository::ProjectOptionsRepository;
use sea_orm::DatabaseConnection;

pub(crate) struct AppState {
    pub(crate) project_options_repository: ProjectOptionsRepository,
    pub(crate) complaints_repository: ComplaintsRepository,
}

impl AppState {
    pub(crate) fn new(db_conn: DatabaseConnection) -> Self {
        Self {
            project_options_repository: ProjectOptionsRepository::new(db_conn.clone()),
            complaints_repository: ComplaintsRepository::new(db_conn.clone()),
        }
    }
}
