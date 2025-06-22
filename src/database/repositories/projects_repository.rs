use crate::database::repository_methods_trait::RepositoryMethods;
use derive_new::new;
use entity::projects::{ActiveModel, Entity};
use entity::{group_members, groups, projects, students};
use repository_macro::RepositoryMethods;
use sea_orm::ColumnTrait;
use sea_orm::QueryFilter;
use sea_orm::{DatabaseConnection, DbErr, EntityTrait, JoinType, QuerySelect, RelationTrait};

#[derive(new, RepositoryMethods, Clone)]
pub(crate) struct ProjectRepository {
    db_conn: DatabaseConnection,
}

impl ProjectRepository {
    pub async fn find_projects_for_student(
        &self, student_id: i32,
    ) -> Result<Vec<projects::Model>, DbErr> {
        let projects: Vec<projects::Model> = projects::Entity::find()
            .distinct()
            .join(JoinType::InnerJoin, projects::Relation::Groups.def())
            .join(JoinType::InnerJoin, groups::Relation::GroupMembers.def())
            .join(JoinType::InnerJoin, group_members::Relation::Students.def())
            .filter(students::Column::StudentId.eq(student_id))
            .into_model::<projects::Model>()
            .all(&self.db_conn)
            .await?;

        Ok(projects)
    }
}
