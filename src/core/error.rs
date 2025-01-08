use std::fmt;
use thiserror::Error;
use sqlx::migrate::MigrateError;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("Migration error: {0}")]
    Migration(#[from] MigrateError),

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("CSV error: {0}")]
    Csv(#[from] csv::Error),

    #[error("Plugin error: {0}")]
    Plugin(String),

    #[error("Invalid operation: {0}")]
    InvalidOperation(String),

    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Configuration error: {0}")]
    Config(String),

    #[error("System error: {0}")]
    System(String),
}

impl From<String> for AppError {
    fn from(s: String) -> Self {
        AppError::System(s)
    }
}

impl From<&str> for AppError {
    fn from(s: &str) -> Self {
        AppError::System(s.to_string())
    }
} 