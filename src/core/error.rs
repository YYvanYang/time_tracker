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
}

pub type AppResult<T> = Result<T, AppError>;

impl From<notify_rust::error::Error> for AppError {
    fn from(err: notify_rust::error::Error) -> Self {
        Self::Platform(err.to_string())
    }
}

impl From<csv::Error> for AppError {
    fn from(err: csv::Error) -> Self {
        Self::Io(std::io::Error::new(std::io::ErrorKind::Other, err.to_string()))
    }
}

impl From<eframe::Error> for AppError {
    fn from(err: eframe::Error) -> Self {
        Self::Platform(err.to_string())
    }
}

impl From<tray_item::TIError> for AppError {
    fn from(err: tray_item::TIError) -> Self {
        Self::Platform(err.to_string())
    }
}

#[derive(Debug)]
pub struct NotFoundError(pub String);

impl From<NotFoundError> for AppError {
    fn from(err: NotFoundError) -> Self {
        Self::Storage(err.0)
    }
} 