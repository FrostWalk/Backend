use derive_new::new;
use sea_orm::DatabaseConnection;
#[derive(new)]
pub struct ComplaintsRepository {
    db_conn: DatabaseConnection,
}

impl ComplaintsRepository {
    /*pub async fn get_all(&self) -> Result<Vec<complaints::Model>, DbErr> {}*/
}