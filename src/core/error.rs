use std::fmt;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("Storage error: {0}")]
    Storage(String),

    #[error("Platform error: {0}")]
    Platform(String),

    #[error("Plugin error: {0}")]
    Plugin(String),

    #[error("Configuration error: {0}")]
    Config(String),

    #[error("Network error: {0}")]
    Network(String),

    #[error("Permission denied: {0}")]
    PermissionDenied(String),

    #[error("Authentication error: {0}")]
    Authentication(String),

    #[error("Validation error: {0}")]
    Validation(String),

    #[error("Resource busy: {0}")]
    ResourceBusy(String),

    #[error("Resource exhausted: {0}")]
    ResourceExhausted(String),

    #[error("Operation timed out: {0}")]
    Timeout(String),

    #[error("Data corruption: {0}")]
    DataCorruption(String),

    #[error("Migration error: {0}")]
    Migration(String),

    #[error("Plugin compatibility error: {0}")]
    PluginCompatibility(String),
}

pub type AppResult<T> = Result<T, AppError>; 