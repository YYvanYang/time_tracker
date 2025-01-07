use crate::core::AppResult;
use chrono::Local;
use log::{Level, LevelFilter, Log, Metadata, Record};
use std::collections::VecDeque;
use std::fs::{File, OpenOptions};
use std::io::Write;
use std::path::PathBuf;
use std::sync::Mutex;

const MAX_IN_MEMORY_LOGS: usize = 1000;

pub struct Logger {
    file: Option<Mutex<File>>,
    recent_logs: Mutex<VecDeque<LogEntry>>,
}

#[derive(Debug, Clone)]
pub struct LogEntry {
    pub timestamp: chrono::DateTime<Local>,
    pub level: Level,
    pub target: String,
    pub message: String,
    pub module_path: Option<String>,
    pub file: Option<String>,
    pub line: Option<u32>,
}

impl Log for Logger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        metadata.level() <= Level::Trace
    }

    fn log(&self, record: &Record) {
        if !self.enabled(record.metadata()) {
            return;
        }

        let entry = LogEntry {
            timestamp: Local::now(),
            level: record.level(),
            target: record.target().to_string(),
            message: format!("{}", record.args()),
            module_path: record.module_path().map(String::from),
            file: record.file().map(String::from),
            line: record.line(),
        };

        if let Ok(mut recent_logs) = self.recent_logs.lock() {
            recent_logs.push_back(entry.clone());
            if recent_logs.len() > MAX_IN_MEMORY_LOGS {
                recent_logs.pop_front();
            }
        }

        if let Some(file) = &self.file {
            if let Ok(mut file) = file.lock() {
                let _ = writeln!(
                    file,
                    "{} [{}] {}: {}",
                    entry.timestamp.format("%Y-%m-%d %H:%M:%S"),
                    entry.level,
                    entry.target,
                    entry.message
                );
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

impl Logger {
    pub fn new(log_file_path: Option<PathBuf>) -> AppResult<Self> {
        let file = if let Some(path) = log_file_path {
            if let Some(parent) = path.parent() {
                std::fs::create_dir_all(parent)?;
            }
            Some(Mutex::new(OpenOptions::new()
                .create(true)
                .append(true)
                .open(path)?))
        } else {
            None
        };

        Ok(Logger {
            file,
            recent_logs: Mutex::new(VecDeque::with_capacity(MAX_IN_MEMORY_LOGS)),
        })
    }

    pub fn init(log_file_path: Option<PathBuf>) -> AppResult<()> {
        let logger = Logger::new(log_file_path)?;
        log::set_boxed_logger(Box::new(logger))?;
        log::set_max_level(LevelFilter::Trace);
        Ok(())
    }

    pub fn get_recent_logs(&self) -> AppResult<Vec<LogEntry>> {
        Ok(self.recent_logs
            .lock()
            .map_err(|_| crate::core::AppError::Config("Failed to lock recent logs".into()))?
            .iter()
            .cloned()
            .collect())
    }

    pub fn get_recent_logs_by_level(&self, level: Level) -> AppResult<Vec<LogEntry>> {
        Ok(self.recent_logs
            .lock()
            .map_err(|_| crate::core::AppError::Config("Failed to lock recent logs".into()))?
            .iter()
            .filter(|entry| entry.level <= level)
            .cloned()
            .collect())
    }
} 