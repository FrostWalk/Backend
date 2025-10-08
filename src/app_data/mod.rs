use crate::config::Config;
use crate::mail::Mailer;
use welds::connections::postgres::PostgresClient;

#[derive(Clone)]
pub(crate) struct AppData {
    pub(crate) config: Config,
    pub(crate) db: PostgresClient,
    pub(crate) mailer: Mailer,
}

impl AppData {
    pub(crate) async fn new(config: Config, db: PostgresClient, mailer: Mailer) -> Self {
        Self { db, config, mailer }
    }
}
