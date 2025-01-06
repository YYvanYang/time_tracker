use thiserror::Error;
use std::sync::PoisonError;
use plotters::drawing::DrawingAreaErrorKind;
use std::error::Error as StdError;

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
    
    #[error("State error: {0}")]
    State(String),
    
    #[error("Lock error: {0}")]
    Lock(String),
    
    #[error("Dialog error: {0}")]
    Dialog(String),

    #[error("Drawing error: {0}")]
    Drawing(String),

    #[error("CSV error: {0}")]
    Csv(#[from] csv::Error),
}

pub type Result<T> = std::result::Result<T, TimeTrackerError>;

impl<E: std::fmt::Debug + StdError + Send + Sync + 'static> From<DrawingAreaErrorKind<E>> for TimeTrackerError {
    fn from(err: DrawingAreaErrorKind<E>) -> Self {
        TimeTrackerError::Drawing(format!("Drawing error: {:?}", err))
    }
}

impl From<chrono::OutOfRangeError> for TimeTrackerError {
    fn from(err: chrono::OutOfRangeError) -> Self {
        TimeTrackerError::Time(format!("Time error: {}", err))
    }
}

impl<T> From<PoisonError<T>> for TimeTrackerError {
    fn from(err: PoisonError<T>) -> Self {
        TimeTrackerError::Lock(err.to_string())
    }
}

impl From<log::SetLoggerError> for TimeTrackerError {
    fn from(err: log::SetLoggerError) -> Self {
        TimeTrackerError::Config(format!("Failed to set logger: {}", err))
    }
}

impl From<r2d2::Error> for TimeTrackerError {
    fn from(err: r2d2::Error) -> Self {
        TimeTrackerError::Database(rusqlite::Error::InvalidParameterName(err.to_string()))
    }
}

impl From<tray_item::TIError> for TimeTrackerError {
    fn from(err: tray_item::TIError) -> Self {
        TimeTrackerError::Platform(format!("Tray error: {}", err))
    }
}

impl From<notify::Error> for TimeTrackerError {
    fn from(err: notify::Error) -> Self {
        TimeTrackerError::Config(format!("File watch error: {}", err))
    }
}