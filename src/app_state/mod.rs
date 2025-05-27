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
use crate::database::repositories::users_repository::StudentsRepository;
use sea_orm::Database;

#[derive(Clone)]
pub(crate) struct AppState {
    pub(crate) repositories: Repositories,
    pub(crate) config: Config,
}
#[derive(Clone)]
pub(crate) struct Repositories {
    pub(crate) project_options: ProjectOptionsRepository,
    pub(crate) complaints: ComplaintsRepository,
    pub(crate) project: ProjectRepository,
    pub(crate) auxiliary_roles: AuxiliaryRolesRepository,
    pub(crate) blacklist: BlacklistRepository,
    pub(crate) fairs: FairsRepository,
    pub(crate) groups: GroupsRepository,
    pub(crate) roles: RolesRepository,
    pub(crate) security_codes: SecurityCodesRepository,
    pub(crate) users_projects_roles: UsersProjectsRolesRepository,
    pub(crate) admins: AdminsRepository,
    pub(crate) students: StudentsRepository,
    // todo Missing repositories
}

impl AppState {
    pub(crate) async fn new(config: Config) -> Self {
        let db_conn = Database::connect(config.db_url()).await.unwrap();
        Self {
            repositories: Repositories {
                project_options: ProjectOptionsRepository::new(db_conn.clone()),
                complaints: ComplaintsRepository::new(db_conn.clone()),
                students: StudentsRepository::new(db_conn.clone()),
                project: ProjectRepository::new(db_conn.clone()),
                auxiliary_roles: AuxiliaryRolesRepository::new(db_conn.clone()),
                blacklist: BlacklistRepository::new(db_conn.clone()),
                fairs: FairsRepository::new(db_conn.clone()),
                groups: GroupsRepository::new(db_conn.clone()),
                roles: RolesRepository::new(db_conn.clone()),
                security_codes: SecurityCodesRepository::new(db_conn.clone()),
                users_projects_roles: UsersProjectsRolesRepository::new(db_conn.clone()),
                admins: AdminsRepository::new(db_conn.clone()),
            },
            config,
        }
    }
}
