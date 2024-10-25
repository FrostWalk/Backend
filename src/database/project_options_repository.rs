use derive_new::new;
use entity::prelude::ProjectOptions;
use entity::project_options;
use sea_orm::{ActiveModelTrait, ColumnTrait, DatabaseConnection, DbErr, DeleteResult, EntityTrait, InsertResult, QueryFilter, QueryOrder};

#[derive(new)]
pub(crate) struct ProjectOptionsRepository {
    db_conn: DatabaseConnection,
}

impl ProjectOptionsRepository {
    pub async fn get_all(&self) -> Result<Vec<project_options::Model>, DbErr> {
        ProjectOptions::find().order_by_asc(project_options::Column::Name).all(&self.db_conn).await
    }

    pub async fn get_from_name(&self, name: &str) -> Result<Option<project_options::Model>, DbErr> {
        ProjectOptions::find().filter(project_options::Column::Name.eq(name)).one(&self.db_conn).await
    }

    pub async fn get_from_id(&self, id: i32) -> Result<Option<project_options::Model>, DbErr> {
        ProjectOptions::find().filter(project_options::Column::Id.eq(id)).one(&self.db_conn).await
    }
    pub async fn create(&self, model: project_options::ActiveModel) -> Result<project_options::Model, DbErr> {
        model.insert(&self.db_conn).await
    }

    pub async fn create_all(&self, models: Vec<project_options::ActiveModel>) -> Result<InsertResult<project_options::ActiveModel>, DbErr> {
        ProjectOptions::insert_many(models.to_vec()).exec(&self.db_conn).await
    }

    pub async fn update(&self, model: project_options::ActiveModel) -> Result<project_options::Model, DbErr> {
        model.update(&self.db_conn).await
    }

    pub async fn delete_by_id(&self, id: i32) -> Result<DeleteResult, DbErr> {
        ProjectOptions::delete_by_id(id).exec(&self.db_conn).await
    }
}
