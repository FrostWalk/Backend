use migration::IntoCondition;
use sea_orm::QueryFilter;
use sea_orm::{
    ActiveModelTrait, DatabaseConnection, DbErr, DeleteResult, EntityTrait, InsertResult,
};
#[allow(dead_code)]
pub(crate) trait RepositoryMethods<E, AM>
where
    E: EntityTrait,
    AM: ActiveModelTrait<Entity = E> + sea_orm::ActiveModelBehavior,
{
    fn db_conn(&self) -> &DatabaseConnection;

    async fn get_all(&self) -> Result<Vec<E::Model>, DbErr> {
        E::find().all(self.db_conn()).await
    }

    async fn get_from_id(&self, id: i32) -> Result<Option<E::Model>, DbErr>
    where
        <<E as EntityTrait>::PrimaryKey as sea_orm::PrimaryKeyTrait>::ValueType: From<i32>,
    {
        E::find_by_id(id).one(self.db_conn()).await
    }
    async fn get_all_from_filter(
        &self, filter: impl IntoCondition,
    ) -> Result<Vec<E::Model>, DbErr> {
        E::find().filter(filter).all(self.db_conn()).await
    }

    async fn get_one_from_filter(
        &self, filter: impl IntoCondition,
    ) -> Result<Option<E::Model>, DbErr> {
        E::find().filter(filter).one(self.db_conn()).await
    }
    async fn create(&self, model: AM) -> Result<InsertResult<AM>, DbErr> {
        E::insert(model).exec(self.db_conn()).await
    }

    async fn create_all(&self, models: Vec<AM>) -> Result<InsertResult<AM>, DbErr> {
        E::insert_many(models).exec(self.db_conn()).await
    }

    async fn delete_from_id(&self, id: i32) -> Result<DeleteResult, DbErr>
    where
        <<E as EntityTrait>::PrimaryKey as sea_orm::PrimaryKeyTrait>::ValueType: From<i32>,
    {
        E::delete_by_id(id).exec(self.db_conn()).await
    }

    async fn update(&self, model: AM) -> Result<E::Model, DbErr>
    where
        <E as EntityTrait>::Model: sea_orm::IntoActiveModel<AM>,
    {
        E::update(model).exec(self.db_conn()).await
    }
}
