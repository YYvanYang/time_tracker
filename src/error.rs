use thiserror::Error;
use std::time::SystemTimeError;

#[derive(Error, Debug)]
pub enum TimeTrackerError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Database error: {0}")]
    Database(#[from] rusqlite::Error),

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("System time error: {0}")]
    SystemTime(#[from] SystemTimeError),

    #[error("CSV error: {0}")]
    Csv(#[from] csv::Error),

    #[error("Platform error: {0}")]
    Platform(String),

    #[error("Configuration error: {0}")]
    Config(String),

    #[error("Storage error: {0}")]
    Storage(String),

    #[error("Invalid time format: {0}")]
    TimeFormat(String),

    #[error("Cache error: {0}")]
    Cache(String),

    #[error("Notification error: {0}")]
    Notification(String),

    #[error("GUI error: {0}")]
    Gui(String),

    #[error("Audio error: {0}")]
    Audio(String),
}

pub type Result<T> = std::result::Result<T, TimeTrackerError>;

// 用于从字符串创建错误
impl From<String> for TimeTrackerError {
    fn from(s: String) -> Self {
        TimeTrackerError::Platform(s)
    }
}

impl From<&str> for TimeTrackerError {
    fn from(s: &str) -> Self {
        TimeTrackerError::Platform(s.to_string())
    }
}

// 辅助方法
impl TimeTrackerError {
    pub fn is_fatal(&self) -> bool {
        matches!(
            self,
            TimeTrackerError::Database(_) | 
            TimeTrackerError::Storage(_) |
            TimeTrackerError::Config(_)
        )
    }

    pub fn should_retry(&self) -> bool {
        matches!(
            self,
            TimeTrackerError::Io(_) |
            TimeTrackerError::Platform(_) |
            TimeTrackerError::Cache(_)
        )
    }
}