use chrono::{DateTime, Local};
use serde::{Deserialize, Serialize};
use std::time::Duration;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Project {
    pub id: Option<i64>,
    pub name: String,
    pub description: String,
    pub created_at: DateTime<Local>,
    pub updated_at: DateTime<Local>,
    pub color: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Activity {
    pub id: Option<i64>,
    pub app_name: String,
    pub window_title: String,
    pub start_time: DateTime<Local>,
    pub duration: Duration,
    pub category: Option<String>,
    pub is_productive: bool,
    pub project_id: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PomodoroSession {
    pub id: Option<i64>,
    pub start_time: DateTime<Local>,
    pub end_time: Option<DateTime<Local>>,
    pub status: PomodoroStatus,
    pub notes: Option<String>,
    pub tags: Vec<String>,
    pub project_id: Option<i64>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PomodoroStatus {
    Running,
    Completed,
    Interrupted,
    Break,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProductivityStats {
    pub total_time: Duration,
    pub productive_time: Duration,
    pub productivity_ratio: f64,
    pub most_productive_hour: Option<u32>,
    pub most_productive_day: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CategoryStats {
    pub category: String,
    pub total_time: Duration,
    pub activity_count: u32,
    pub average_duration: Duration,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppUsageRecord {
    pub id: Option<i64>,
    pub app_name: String,
    pub window_title: String,
    pub start_time: DateTime<Local>,
    pub duration: Duration,
    pub category: Option<String>,
    pub is_productive: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PomodoroRecord {
    pub id: Option<i64>,
    pub start_time: DateTime<Local>,
    pub end_time: DateTime<Local>,
    pub status: PomodoroStatus,
    pub notes: Option<String>,
    pub tags: Vec<String>,
    pub project_id: Option<i64>,
}

impl Default for PomodoroStatus {
    fn default() -> Self {
        Self::Running
    }
}

impl std::fmt::Display for PomodoroStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PomodoroStatus::Running => write!(f, "running"),
            PomodoroStatus::Completed => write!(f, "completed"),
            PomodoroStatus::Interrupted => write!(f, "interrupted"),
            PomodoroStatus::Break => write!(f, "break"),
        }
    }
}

impl std::str::FromStr for PomodoroStatus {
    type Err = crate::AppError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "running" => Ok(Self::Running),
            "completed" => Ok(Self::Completed),
            "interrupted" => Ok(Self::Interrupted),
            "break" => Ok(Self::Break),
            _ => Err(crate::AppError::InvalidInput(format!(
                "Invalid pomodoro status: {}",
                s
            ))),
        }
    }
} 