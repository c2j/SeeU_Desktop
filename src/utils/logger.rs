use log::{LevelFilter, Log, Metadata, Record};
use std::fs::{File, OpenOptions};
use std::io::Write;
use std::path::PathBuf;
use std::sync::Mutex;
use crate::platform;

/// Custom logger for the application
pub struct Logger {
    level: LevelFilter,
    file: Option<Mutex<File>>,
}

impl Logger {
    /// Create a new logger
    pub fn new(level: LevelFilter, log_to_file: bool) -> Self {
        let file = if log_to_file {
            let log_path = Self::log_path();

            // Create parent directories if they don't exist
            if let Some(parent) = log_path.parent() {
                std::fs::create_dir_all(parent).ok();
            }

            // Open log file
            let file = OpenOptions::new()
                .create(true)
                .append(true)
                .open(log_path)
                .ok();

            file.map(Mutex::new)
        } else {
            None
        };

        Self { level, file }
    }

    /// Initialize the logger
    pub fn init(level: LevelFilter, log_to_file: bool) -> Result<(), log::SetLoggerError> {
        let logger = Box::new(Self::new(level, log_to_file));
        log::set_boxed_logger(logger)?;
        log::set_max_level(level);
        Ok(())
    }

    /// Get the path to the log file
    pub fn log_path() -> PathBuf {
        let mut path = platform::app_log_dir();

        // Create a log file with the current date
        let now = chrono::Local::now();
        let filename = format!("seeu_{}.log", now.format("%Y-%m-%d"));
        path.push(filename);

        path
    }
}

impl Log for Logger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        metadata.level() <= self.level
    }

    fn log(&self, record: &Record) {
        if self.enabled(record.metadata()) {
            let now = chrono::Local::now();
            let message = format!(
                "[{} {} {}:{}] {}\n",
                now.format("%Y-%m-%d %H:%M:%S"),
                record.level(),
                record.file().unwrap_or("unknown"),
                record.line().unwrap_or(0),
                record.args()
            );

            // Print to stdout
            print!("{}", message);

            // Write to file if enabled
            if let Some(file) = &self.file {
                if let Ok(mut file) = file.lock() {
                    let _ = file.write_all(message.as_bytes());
                }
            }
        }
    }

    fn flush(&self) {
        if let Some(file) = &self.file {
            if let Ok(mut file) = file.lock() {
                let _ = file.flush();
            }
        }
    }
}