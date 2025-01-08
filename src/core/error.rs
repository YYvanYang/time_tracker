use thiserror::Error;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("Resource not found: {0}")]
    NotFound(String),

    #[error("CSV error: {0}")]
    Csv(#[from] csv::Error),

    #[error("CSV writer error: {0}")]
    CsvWriter(#[from] csv::IntoInnerError<csv::Writer<Vec<u8>>>),

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("UTF-8 error: {0}")]
    Utf8(#[from] std::string::FromUtf8Error),

    #[error("Plugin error: {0}")]
    Plugin(#[from] libloading::Error),

    #[error("GUI error: {0}")]
    Gui(#[from] eframe::Error),

    #[error("Notification error: {0}")]
    Notification(#[from] notify_rust::error::Error),

    #[error("Platform error: {0}")]
    Platform(String),

    #[error("Invalid operation: {0}")]
    InvalidOperation(String),

    #[error("Storage error: {0}")]
    Storage(String),

    #[error("Validation error: {0}")]
    Validation(String),
}

pub type AppResult<T> = Result<T, AppError>;

impl From<tray_item::TIError> for AppError {
    fn from(err: tray_item::TIError) -> Self {
        Self::Platform(err.to_string())
    }
}

pub struct NotFoundError(pub String);

impl From<NotFoundError> for AppError {
    fn from(err: NotFoundError) -> Self {
        Self::NotFound(err.0)
    }
} 