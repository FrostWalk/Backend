use crate::config::Config;
use welds::connections::postgres::PostgresClient;

#[derive(Clone)]
pub(crate) struct AppData {
    pub(crate) config: Config,
    pub(crate) db: PostgresClient,
}

impl AppData {
    pub(crate) async fn new(config: Config, db: PostgresClient) -> Self {
        Self { db, config }
    }
}
