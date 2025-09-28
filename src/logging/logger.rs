use crate::logging::context::{get_request_context, init_request_context_storage};
use crate::logging::model::LogEntry;
use chrono::Utc;
use log::{Level, LevelFilter, Metadata, Record};
use mongodb::{Client, Collection, Database};
use tokio::sync::mpsc;
use uuid::Uuid;

pub struct MongoLogger {
    sender: mpsc::UnboundedSender<LogEntry>,
}

impl log::Log for MongoLogger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        metadata.level() <= Level::Info
    }

    fn log(&self, record: &Record) {
        if self.enabled(record.metadata()) {
            // Print to console first
            let timestamp = Utc::now().format("%Y-%m-%d %H:%M:%S%.3f UTC");
            let level_str = match record.level() {
                Level::Error => "\x1b[31mERROR\x1b[0m", // Red
                Level::Warn => "\x1b[33mWARN\x1b[0m",   // Yellow
                Level::Info => "\x1b[32mINFO\x1b[0m",   // Green
                Level::Debug => "\x1b[36mDEBUG\x1b[0m", // Cyan
                Level::Trace => "\x1b[37mTRACE\x1b[0m", // White
            };

            println!(
                "[{}] {} [{}] {}",
                timestamp,
                level_str,
                record.target(),
                record.args()
            );

            // Create entry for MongoDB
            let entry = LogEntry {
                id: Uuid::new_v4(),
                timestamp: Utc::now(),
                level: record.level().to_string(),
                message: record.args().to_string(),
                target: record.target().to_string(),
                module_path: record.module_path().map(String::from),
                file: record.file().map(String::from),
                line: record.line(),
                request_context: get_request_context(),
            };

            let _ = self.sender.send(entry);
        }
    }

    fn flush(&self) {}
}

async fn log_writer_task(
    mut receiver: mpsc::UnboundedReceiver<LogEntry>, collection: Collection<LogEntry>,
) {
    while let Some(entry) = receiver.recv().await {
        if let Err(e) = collection.insert_one(entry).await {
            eprintln!("Failed to insert log entry: {}", e);
        }
    }
}

pub async fn init_mongo_logger(
    mongodb_uri: &str, db_name: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    // Initialize request context storage
    init_request_context_storage();

    let client = Client::with_uri_str(mongodb_uri).await?;
    let db: Database = client.database(db_name);
    let collection: Collection<LogEntry> = db.collection("logs");

    let (sender, receiver) = mpsc::unbounded_channel();

    let collection_clone = collection.clone();
    tokio::spawn(async move {
        log_writer_task(receiver, collection_clone).await;
    });

    let logger = MongoLogger { sender };

    log::set_boxed_logger(Box::new(logger))?;
    log::set_max_level(LevelFilter::Info);

    Ok(())
}
