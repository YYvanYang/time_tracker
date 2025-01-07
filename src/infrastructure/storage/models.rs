// src/storage/models.rs

use chrono::{DateTime, Local};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Project {
    pub id: Option<i64>,
    pub name: String,
    pub description: Option<String>,
    pub created_at: DateTime<Local>,
    pub updated_at: DateTime<Local>,
    pub color: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Category {
    pub id: Option<i64>,
    pub name: String,
    pub description: Option<String>,
    pub productivity_score: f64,
    pub created_at: DateTime<Local>,
    pub updated_at: DateTime<Local>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Tag {
    pub id: Option<i64>,
    pub name: String,
    pub created_at: DateTime<Local>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageHealth {
    pub is_healthy: bool,
    pub database_size: u64,
    pub app_usage_count: u64,
    pub pomodoro_count: u64,
    pub last_backup: Option<DateTime<Local>>,
    pub needs_vacuum: bool,
}