use crate::database::repository_methods::RepositoryMethods;
use derive_new::new;
use entity::prelude::{Projects, Roles};
use entity::projects::{ActiveModel, Entity};
use entity::users_projects_and_roles::Column;
use entity::{projects, roles, users_projects_and_roles};
use repository_macro::RepositoryMethods;
use sea_orm::prelude::DateTime;
use sea_orm::{
    ColumnTrait, DatabaseConnection, DbErr, EntityTrait, LoaderTrait, QueryFilter, QueryOrder,
};

#[derive(new, RepositoryMethods, Clone)]
pub(crate) struct ProjectRepository {
    db_conn: DatabaseConnection,
}

pub(crate) type ProjectsAndRoles = Vec<(projects::Model, roles::Model, (bool, Option<DateTime>))>;

impl ProjectRepository {
    pub(crate) async fn get_user_projects(&self, user_id: i32) -> Result<ProjectsAndRoles, DbErr> {
        // Find all project-role relationships for the user
        let users_projects_roles = users_projects_and_roles::Entity::find()
            .filter(Column::UserId.eq(user_id))
            .order_by_desc(Column::ProjectId)
            .all(&self.db_conn)
            .await?;

        if users_projects_roles.is_empty() {
            return Ok(vec![]);
        }

        let projects = users_projects_roles
            .load_one(Projects, &self.db_conn)
            .await?;
        let roles = users_projects_roles.load_one(Roles, &self.db_conn).await?;

        let mut result: ProjectsAndRoles = Vec::with_capacity(users_projects_roles.len());

        for i in 0..users_projects_roles.len() {
            result[i] = (
                projects[i].clone().unwrap(),
                roles[i].clone().unwrap(),
                (
                    users_projects_roles[i].has_retired,
                    users_projects_roles[i].retirement_date,
                ),
            )
        }

        Ok(result)
    }
}
