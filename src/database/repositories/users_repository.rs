use crate::database::repository_methods_trait::RepositoryMethods;
use crate::jwt::role::UserRole;
use derive_new::new;
use entity::users::Entity;
use entity::users::{ActiveModel, Model};
use entity::{projects, users, users_projects_and_roles};
use repository_macro::RepositoryMethods;
use sea_orm::{ColumnTrait, JoinType, QuerySelect, RelationTrait};
use sea_orm::{DatabaseConnection, DbErr, EntityTrait, QueryFilter, QueryOrder};

#[derive(new, RepositoryMethods, Clone)]
pub(crate) struct UsersRepository {
    db_conn: DatabaseConnection,
}

impl UsersRepository {
    pub(crate) async fn get_current_role(&self, user_id: i32) -> Result<Option<UserRole>, DbErr> {
        let model = users_projects_and_roles::Entity::find()
            .filter(users_projects_and_roles::Column::UserId.eq(user_id))
            .join(
                JoinType::InnerJoin,
                users_projects_and_roles::Relation::Projects.def(),
            )
            .order_by_desc(projects::Column::Year)
            .select_only()
            .column(users_projects_and_roles::Column::RoleId)
            .limit(1)
            .one(&self.db_conn)
            .await?;

        Ok(model.map(|m| m.role_id.into()))
    }

    pub(crate) async fn get_from_mail(&self, mail: &String) -> Result<Option<Model>, DbErr> {
        self.get_one_from_filter(users::Column::Name.eq(mail)).await
    }
}
