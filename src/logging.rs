use crate::error::Result;
use chrono::Local;
use log::{Level, LevelFilter, Log, Metadata, Record, SetLoggerError};
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

impl Logger {
    pub fn init(log_path: Option<PathBuf>) -> std::result::Result<(), SetLoggerError> {
        let file = log_path.map(|path| {
            if let Some(parent) = path.parent() {
                std::fs::create_dir_all(parent).expect("Failed to create log directory");
            }
            let file = OpenOptions::new()
                .create(true)
                .append(true)
                .open(path)
                .expect("Failed to open log file");
            Mutex::new(file)
        });

        let logger = Box::new(Logger {
            file,
            recent_logs: Mutex::new(VecDeque::with_capacity(MAX_IN_MEMORY_LOGS)),
        });

        log::set_boxed_logger(logger)?;
        log::set_max_level(LevelFilter::Info);
        Ok(())
    }

    pub fn get_recent_logs(&self) -> Vec<LogEntry> {
        self.recent_logs
            .lock()
            .expect("Failed to lock recent_logs")
            .iter()
            .cloned()
            .collect()
    }

    pub fn get_recent_logs_by_level(&self, level: Level) -> Vec<LogEntry> {
        self.recent_logs
            .lock()
            .expect("Failed to lock recent_logs")
            .iter()
            .filter(|entry| entry.level == level)
            .cloned()
            .collect()
    }

    fn add_log_entry(&self, entry: LogEntry) {
        let mut recent_logs = self.recent_logs.lock().expect("Failed to lock recent_logs");
        if recent_logs.len() >= MAX_IN_MEMORY_LOGS {
            recent_logs.pop_front();
        }
        recent_logs.push_back(entry);
    }
}

impl Log for Logger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        metadata.level() <= log::max_level()
    }

    fn log(&self, record: &Record) {
        if self.enabled(record.metadata()) {
            let entry = LogEntry {
                timestamp: Local::now(),
                level: record.level(),
                target: record.target().to_string(),
                message: record.args().to_string(),
                module_path: record.module_path().map(String::from),
                file: record.file().map(String::from),
                line: record.line(),
            };

            // 添加到内存缓存
            self.add_log_entry(entry.clone());

            // 格式化日志消息
            let formatted_log = format!(
                "[{} {} {} {}:{}] {}\n",
                entry.timestamp.format("%Y-%m-%d %H:%M:%S"),
                entry.level,
                entry.target,
                entry.file.unwrap_or_else(|| "unknown".to_string()),
                entry.line.unwrap_or(0),
                entry.message
            );

            // 写入文件（如果配置了文件日志）
            if let Some(file) = &self.file {
                if let Ok(mut file) = file.lock() {
                    let _ = file.write_all(formatted_log.as_bytes());
                    let _ = file.flush();
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

// 用于在GUI中显示日志的工具函数
pub fn format_log_entry(entry: &LogEntry) -> String {
    format!(
        "{} [{}] {}",
        entry.timestamp.format("%H:%M:%S"),
        entry.level,
        entry.message
    )
}

// 用于获取日志文件路径的辅助函数
pub fn get_default_log_path() -> PathBuf {
    let log_dir = dirs::data_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("time_tracker")
        .join("logs");

    log_dir.join(format!(
        "time_tracker_{}.log",
        Local::now().format("%Y-%m-%d")
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_logger_initialization() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let log_path = temp_dir.path().join("test.log");
        
        // 初始化日志系统
        Logger::init(Some(log_path.clone()))?;

        // 写入一些日志
        log::info!("Test log message");
        log::warn!("Test warning message");
        log::error!("Test error message");

        // 验证日志文件是否创建并包含内容
        let log_content = std::fs::read_to_string(log_path)?;
        assert!(log_content.contains("Test log message"));
        assert!(log_content.contains("Test warning message"));
        assert!(log_content.contains("Test error message"));

        Ok(())
    }

    #[test]
    fn test_recent_logs() -> Result<()> {
        let logger = Logger {
            file: None,
            recent_logs: Mutex::new(VecDeque::new()),
        };

        // 添加一些测试日志
        for i in 0..MAX_IN_MEMORY_LOGS + 10 {
            logger.log(&Record::builder()
                .args(format_args!("Test message {}", i))
                .level(Level::Info)
                .target("test")
                .build());
        }

        // 验证日志数量限制
        let recent_logs = logger.get_recent_logs();
        assert_eq!(recent_logs.len(), MAX_IN_MEMORY_LOGS);

        // 验证最新的日志在最后
        let last_log = recent_logs.last().unwrap();
        assert!(last_log.message.contains(&format!("Test message {}", MAX_IN_MEMORY_LOGS + 9)));

        Ok(())
    }

    #[test]
    fn test_log_levels() -> Result<()> {
        let logger = Logger {
            file: None,
            recent_logs: Mutex::new(VecDeque::new()),
        };

        // 添加不同级别的日志
        logger.log(&Record::builder()
            .args(format_args!("Info message"))
            .level(Level::Info)
            .target("test")
            .build());

        logger.log(&Record::builder()
            .args(format_args!("Error message"))
            .level(Level::Error)
            .target("test")
            .build());

        // 检查按级别过滤
        let error_logs = logger.get_recent_logs_by_level(Level::Error);
        assert_eq!(error_logs.len(), 1);
        assert!(error_logs[0].message.contains("Error message"));

        Ok(())
    }
}