use migration::IntoCondition;
use sea_orm::QueryFilter;
use sea_orm::{
    ActiveModelTrait, DatabaseConnection, DbErr, DeleteResult, EntityTrait, InsertResult,
};
#[allow(dead_code)]
/// RepositoryMethods is a trait providing common database operations for entities and their active models.
/// It abstracts away the database connection and provides standard CRUD (Create, Read, Update, Delete) operations.
///
/// # Type Parameters
/// - `E`: Entity type implementing `EntityTrait`
/// - `AM`: ActiveModel type implementing `ActiveModelTrait` for the entity
pub(crate) trait RepositoryMethods<E, AM>
where
    E: EntityTrait,
    AM: ActiveModelTrait<Entity = E> + sea_orm::ActiveModelBehavior,
{
    /// Returns a reference to the database connection
    fn db_conn(&self) -> &DatabaseConnection;

    /// Retrieves all records of the entity from the database
    ///
    /// # Returns
    /// - `Result<Vec<E::Model>, DbErr>`: A vector of model instances or an error
    async fn get_all(&self) -> Result<Vec<E::Model>, DbErr> {
        E::find().all(self.db_conn()).await
    }

    /// Retrieves a single entity by its ID
    ///
    /// # Parameters
    /// - `id`: The ID of the entity to retrieve
    ///
    /// # Returns
    /// - `Result<Option<E::Model>, DbErr>`: Optional model instance or error
    async fn get_from_id(&self, id: i32) -> Result<Option<E::Model>, DbErr>
    where
        <<E as EntityTrait>::PrimaryKey as sea_orm::PrimaryKeyTrait>::ValueType: From<i32>,
    {
        E::find_by_id(id).one(self.db_conn()).await
    }

    /// Retrieves all entities that match the given filter condition
    ///
    /// # Parameters
    /// - `filter`: Filter condition implementing IntoCondition trait
    async fn get_all_from_filter(
        &self, filter: impl IntoCondition,
    ) -> Result<Vec<E::Model>, DbErr> {
        E::find().filter(filter).all(self.db_conn()).await
    }

    /// Retrieves a single entity that matches the given filter condition
    ///
    /// # Parameters
    /// - `filter`: Filter condition implementing IntoCondition trait
    async fn get_one_from_filter(
        &self, filter: impl IntoCondition,
    ) -> Result<Option<E::Model>, DbErr> {
        E::find().filter(filter).one(self.db_conn()).await
    }

    /// Creates a new entity from the provided active model
    ///
    /// # Parameters
    /// - `model`: Active model to insert into database
    async fn create(&self, model: AM) -> Result<InsertResult<AM>, DbErr> {
        E::insert(model).exec(self.db_conn()).await
    }

    /// Creates multiple entities from a vector of active models
    ///
    /// # Parameters
    /// - `models`: Vector of active models to insert into database
    async fn create_all(&self, models: Vec<AM>) -> Result<InsertResult<AM>, DbErr> {
        E::insert_many(models).exec(self.db_conn()).await
    }

    /// Deletes an entity by its ID
    ///
    /// # Parameters
    /// - `id`: The ID of the entity to delete
    async fn delete_from_id(&self, id: i32) -> Result<DeleteResult, DbErr>
    where
        <<E as EntityTrait>::PrimaryKey as sea_orm::PrimaryKeyTrait>::ValueType: From<i32>,
    {
        E::delete_by_id(id).exec(self.db_conn()).await
    }

    /// Updates an entity from the provided active model
    ///
    /// # Parameters
    /// - `model`: Active model containing updated values
    async fn update(&self, model: AM) -> Result<E::Model, DbErr>
    where
        <E as EntityTrait>::Model: sea_orm::IntoActiveModel<AM>,
    {
        E::update(model).exec(self.db_conn()).await
    }
}
