use crate::config::Config;
use crate::database::complaints_repository::ComplaintsRepository;
use crate::database::project_options_repository::ProjectOptionsRepository;
use crate::database::users_repository::UsersRepository;
use sea_orm::Database;

pub(crate) struct AppState {
    pub(crate) repositories: Repositories,
    pub(crate) config: Config,
}

pub(crate) struct Repositories {
    pub(crate) project_options_repository: ProjectOptionsRepository,
    pub(crate) complaints_repository: ComplaintsRepository,
    pub(crate) users_repository: UsersRepository,
    // todo Missing repositories
}

impl AppState {
    pub(crate) async fn new(config: Config) -> Self {
        let db_conn = Database::connect(config.db_url()).await.unwrap();
        Self {
            repositories: Repositories {
                project_options_repository: ProjectOptionsRepository::new(db_conn.clone()),
                complaints_repository: ComplaintsRepository::new(db_conn.clone()),
                users_repository: UsersRepository::new(db_conn.clone()),
            },
            config,
        }
    }
}
