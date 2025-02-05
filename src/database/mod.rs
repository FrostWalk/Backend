use migration::{Migrator, MigratorTrait};
use sea_orm::Database;

pub(crate) mod repositories;

pub(crate) mod repository_methods_trait;

#[inline(always)]
pub(super) async fn migrate_database(db_url: &String) {
    let conn = match Database::connect(db_url).await {
        Ok(c) => c,
        Err(e) => {
            panic!("Unable to connect do database.\n{}", e)
        }
    };

    match Migrator::up(&conn, None).await {
        Ok(_) => {}
        Err(e) => {
            panic!("Unable to migrate database.\n{}", e)
        }
    }
}
