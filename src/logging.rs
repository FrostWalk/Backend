use chrono::Utc;
use log::{Level, LevelFilter, Metadata, Record, SetLoggerError};

struct ConsoleLogger;

impl log::Log for ConsoleLogger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        metadata.level() <= Level::Info
    }

    fn log(&self, record: &Record) {
        if !self.enabled(record.metadata()) {
            return;
        }

        let timestamp = Utc::now().format("%Y-%m-%d %H:%M:%S%.3f UTC");
        let level = match record.level() {
            Level::Error => "\x1b[31mERROR\x1b[0m",
            Level::Warn => "\x1b[33mWARN\x1b[0m",
            Level::Info => "\x1b[32mINFO\x1b[0m",
            Level::Debug => "\x1b[36mDEBUG\x1b[0m",
            Level::Trace => "\x1b[37mTRACE\x1b[0m",
        };

        println!(
            "[{}] {} [{}] {}",
            timestamp,
            level,
            record.target(),
            record.args()
        );
    }

    fn flush(&self) {}
}

pub(crate) fn init_console_logger() -> Result<(), SetLoggerError> {
    log::set_boxed_logger(Box::new(ConsoleLogger))?;
    log::set_max_level(LevelFilter::Info);
    Ok(())
}
