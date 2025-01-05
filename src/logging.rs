use crate::error::Result;
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
        let mut recent_logs = self.recent_logs.lock().unwrap();
        recent_logs.push_back(LogEntry {
            timestamp: Local::now(),
            level: record.level(),
            target: record.target().to_string(),
            message: format!("{}", record.args()),
            module_path: record.module_path().map(|s| s.to_string()),
            file: record.file().map(|s| s.to_string()),
            line: record.line(),
        });
        if recent_logs.len() > MAX_IN_MEMORY_LOGS {
            recent_logs.pop_front();
        }

        if let Some(mut file) = self.file.as_ref().map(|f| f.lock().unwrap()) {
            let _ = writeln!(
                file,
                "{} [{}] {}: {}",
                Local::now().format("%Y-%m-%d %H:%M:%S"),
                record.level(),
                record.target(),
                record.args()
            );
        }
    }

    fn flush(&self) {
        if let Some(file) = &self.file {
            let _ = file.lock().unwrap().flush();
        }
    }
}

impl Logger {
    pub fn new(log_file_path: Option<PathBuf>) -> Result<Self> {
        let file = log_file_path.map(|path| {
            OpenOptions::new()
                .create(true)
                .append(true)
                .open(path)
                .map(Mutex::new)
        }).transpose()?;

        Ok(Logger {
            file,
            recent_logs: Mutex::new(VecDeque::new()),
        })
    }

    pub fn init(log_file_path: Option<PathBuf>) -> Result<()> {
        let logger = Logger::new(log_file_path)?;
        log::set_boxed_logger(Box::new(logger))?;
        log::set_max_level(LevelFilter::Trace);
        Ok(())
    }

    pub fn get_recent_logs(&self) -> Vec<LogEntry> {
        self.recent_logs.lock().unwrap().clone().into_iter().collect()
    }

    pub fn get_recent_logs_by_level(&self, level: Level) -> Vec<LogEntry> {
        self.recent_logs.lock().unwrap().iter()
            .filter(|entry| entry.level <= level)
            .cloned()
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_logger_in_memory() -> Result<()> {
        let logger = Logger::new(None)?;

        // 添加一些日志
        log::set_boxed_logger(Box::new(logger)).unwrap();
        log::set_max_level(LevelFilter::Info);

        log::info!("Test info message");
        log::error!("Test error message");

        let recent_logs = logger.get_recent_logs();
        assert_eq!(recent_logs.len(), 2);
        assert_eq!(recent_logs[0].level, Level::Info);
        assert_eq!(recent_logs[1].level, Level::Error);

        Ok(())
    }

    #[test]
    fn test_logger_to_file() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let log_path = temp_dir.path().join("test.log");
        Logger::init(Some(log_path.clone()))?;

        log::info!("Log message to file");

        let contents = std::fs::read_to_string(log_path)?;
        assert!(contents.contains("Log message to file"));

        Ok(())
    }

    #[test]
    fn test_log_levels() -> Result<()> {
        let logger = Logger::new(None)?;
        log::set_boxed_logger(Box::new(logger)).unwrap();
        log::set_max_level(LevelFilter::Warn);

        log::info!("This should not be recorded in memory with Warn level");
        log::warn!("This should be recorded");
        log::error!("This should also be recorded");

        let recent_logs = logger.get_recent_logs();
        assert_eq!(recent_logs.len(), 2);
        assert_eq!(recent_logs[0].level, Level::Warn);
        assert_eq!(recent_logs[1].level, Level::Error);

        Ok(())
    }

    #[test]
    fn test_max_in_memory_logs() -> Result<()> {
        let logger = Logger::new(None)?;
        log::set_boxed_logger(Box::new(logger)).unwrap();
        log::set_max_level(LevelFilter::Info);

        for i in 0..MAX_IN_MEMORY_LOGS + 50 {
            log::info!("Log entry {}", i);
        }

        let recent_logs = logger.get_recent_logs();
        assert_eq!(recent_logs.len(), MAX_IN_MEMORY_LOGS);
        assert!(recent_logs.first().unwrap().message.contains(&(50_usize).to_string())); // 检查最早的日志是否是第 50 条

        Ok(())
    }

    #[test]
    fn test_log_filtering() -> Result<()> {
        let logger = Logger::new(None)?;
        log::set_boxed_logger(Box::new(logger)).unwrap();
        log::set_max_level(LevelFilter::Trace);

        logger.log(
            &Record::builder()
                .args(format_args!("Debug message"))
                .level(Level::Debug)
                .target("test")
                .build()
        );
        logger.log(
            &Record::builder()
                .args(format_args!("Info message"))
                .level(Level::Info)
                .target("test")
                .build()
        );
        logger.log(
            &Record::builder()
                .args(format_args!("Error message"))
                .level(Level::Error)
                .target("test")
                .build()
        );

        let error_logs = logger.get_recent_logs_by_level(Level::Error);
        assert_eq!(error_logs.len(), 1);
        assert!(error_logs[0].message.contains("Error message"));

        let info_and_above_logs = logger.get_recent_logs_by_level(Level::Info);
        assert_eq!(info_and_above_logs.len(), 2);
        assert!(info_and_above_logs.iter().any(|log| log.message.contains("Info message")));
        assert!(info_and_above_logs.iter().any(|log| log.message.contains("Error message")));

        Ok(())
    }
}