use crate::config::Config;
use crate::database::repositories::admins_repository::AdminsRepository;
use crate::database::repositories::auxiliary_roles_repository::AuxiliaryRolesRepository;
use crate::database::repositories::blacklist_repository::BlacklistRepository;
use crate::database::repositories::complaints_repository::ComplaintsRepository;
use crate::database::repositories::fairs_repository::FairsRepository;
use crate::database::repositories::groups_repository::GroupsRepository;
use crate::database::repositories::project_options_repository::ProjectOptionsRepository;
use crate::database::repositories::projects_repository::ProjectRepository;
use crate::database::repositories::roles_repository::RolesRepository;
use crate::database::repositories::security_codes_repository::SecurityCodesRepository;
use crate::database::repositories::users_projects_roles_repository::UsersProjectsRolesRepository;
use crate::database::repositories::users_repository::UsersRepository;
use sea_orm::Database;

#[derive(Clone)]
pub(crate) struct AppState {
    pub(crate) repositories: Repositories,
    pub(crate) config: Config,
}
#[derive(Clone)]
pub(crate) struct Repositories {
    pub(crate) project_options_repository: ProjectOptionsRepository,
    pub(crate) complaints_repository: ComplaintsRepository,
    pub(crate) users_repository: UsersRepository,
    pub(crate) project_repository: ProjectRepository,
    pub(crate) auxiliary_roles_repository: AuxiliaryRolesRepository,
    pub(crate) blacklist_repository: BlacklistRepository,
    pub(crate) fairs_repository: FairsRepository,
    pub(crate) groups_repository: GroupsRepository,
    pub(crate) roles_repository: RolesRepository,
    pub(crate) security_codes_repository: SecurityCodesRepository,
    pub(crate) users_projects_roles_repository: UsersProjectsRolesRepository,
    pub(crate) admins_repo: AdminsRepository,
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
                project_repository: ProjectRepository::new(db_conn.clone()),
                auxiliary_roles_repository: AuxiliaryRolesRepository::new(db_conn.clone()),
                blacklist_repository: BlacklistRepository::new(db_conn.clone()),
                fairs_repository: FairsRepository::new(db_conn.clone()),
                groups_repository: GroupsRepository::new(db_conn.clone()),
                roles_repository: RolesRepository::new(db_conn.clone()),
                security_codes_repository: SecurityCodesRepository::new(db_conn.clone()),
                users_projects_roles_repository: UsersProjectsRolesRepository::new(db_conn.clone()),
                admins_repo: AdminsRepository::new(db_conn.clone()),
            },
            config,
        }
    }
}
