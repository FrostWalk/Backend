use sea_orm_migration::prelude::*;

/// Only for debug purpose, use
/// ```bash
/// sea-orm-cli migrate up
/// ```
#[async_std::main]
async fn main() {
    cli::run_cli(migration::Migrator).await;
}
