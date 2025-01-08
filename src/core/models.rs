use chrono::{DateTime, Local};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Activity {
    pub id: Option<i64>,
    pub name: String,
    pub start_time: DateTime<Local>,
    pub end_time: Option<DateTime<Local>>,
    pub project_id: Option<i64>,
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
pub struct Project {
    pub id: Option<i64>,
    pub name: String,
    pub description: Option<String>,
    pub created_at: DateTime<Local>,
    pub updated_at: DateTime<Local>,
}

impl std::fmt::Display for Project {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PomodoroSession {
    pub id: Option<i64>,
    pub start_time: DateTime<Local>,
    pub end_time: Option<DateTime<Local>>,
    pub duration: i32,
    pub status: PomodoroStatus,
    pub project_id: Option<i64>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PomodoroStatus {
    Work,
    ShortBreak,
    LongBreak,
    Completed,
    Interrupted,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tag {
    pub id: Option<i64>,
    pub name: String,
    pub color: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppState {
    pub current_activity: Option<Activity>,
    pub current_pomodoro: Option<PomodoroSession>,
    pub is_tracking: bool,
    pub last_update: DateTime<Local>,
} 