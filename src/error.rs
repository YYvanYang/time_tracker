use thiserror::Error;
use chrono::OutOfRangeError;

#[derive(Error, Debug)]
pub enum TimeTrackerError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    
    #[error("Database error: {0}")]
    Database(#[from] rusqlite::Error),
    
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
    
    #[error("Storage error: {0}")]
    Storage(String),
    
    #[error("Platform error: {0}")]
    Platform(String),
    
    #[error("GUI error: {0}")]
    Gui(String),
    
    #[error("Config error: {0}")]
    Config(String),
    
    #[error("Time error: {0}")]
    Time(String),
}

pub type Result<T> = std::result::Result<T, TimeTrackerError>;

impl From<chrono::OutOfRangeError> for TimeTrackerError {
    fn from(err: chrono::OutOfRangeError) -> Self {
        TimeTrackerError::Time(format!("Time error: {}", err))
    }
}