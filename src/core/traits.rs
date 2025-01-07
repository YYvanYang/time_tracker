use crate::core::{AppResult, models::*};
use async_trait::async_trait;
use chrono::{DateTime, Local};
use std::time::Duration;

#[async_trait]
pub trait Storage: Send + Sync {
    async fn save_activity(&self, activity: &Activity) -> AppResult<i64>;
    async fn get_activities(&self, start: DateTime<Local>, end: DateTime<Local>) -> AppResult<Vec<Activity>>;
    async fn get_daily_activities(&self) -> AppResult<Vec<Activity>>;
    async fn get_weekly_activities(&self) -> AppResult<Vec<Activity>>;
    async fn delete_activity(&self, id: i64) -> AppResult<()>;
}

#[async_trait]
pub trait ProjectStorage: Send + Sync {
    async fn save_project(&self, project: &Project) -> AppResult<i64>;
    async fn get_project(&self, id: i64) -> AppResult<Option<Project>>;
    async fn get_projects(&self) -> AppResult<Vec<Project>>;
    async fn update_project(&self, project: &Project) -> AppResult<()>;
    async fn delete_project(&self, id: i64) -> AppResult<()>;
}

#[async_trait]
pub trait PomodoroStorage: Send + Sync {
    async fn save_session(&self, session: &PomodoroSession) -> AppResult<i64>;
    async fn get_session(&self, id: i64) -> AppResult<Option<PomodoroSession>>;
    async fn get_sessions(&self, start: DateTime<Local>, end: DateTime<Local>) -> AppResult<Vec<PomodoroSession>>;
    async fn update_session(&self, session: &PomodoroSession) -> AppResult<()>;
    async fn delete_session(&self, id: i64) -> AppResult<()>;
}

#[async_trait]
pub trait TimeTracker: Send + Sync {
    async fn start_tracking(&self) -> AppResult<()>;
    async fn stop_tracking(&self) -> AppResult<()>;
    async fn is_tracking(&self) -> bool;
    async fn get_current_activity(&self) -> AppResult<Option<Activity>>;
    async fn get_elapsed_time(&self) -> Duration;
}

#[async_trait]
pub trait PomodoroTimer: Send + Sync {
    async fn start_pomodoro(&self) -> AppResult<()>;
    async fn stop_pomodoro(&self) -> AppResult<()>;
    async fn pause_pomodoro(&self) -> AppResult<()>;
    async fn resume_pomodoro(&self) -> AppResult<()>;
    async fn skip_break(&self) -> AppResult<()>;
    async fn get_remaining_time(&self) -> Duration;
    async fn get_current_session(&self) -> AppResult<Option<PomodoroSession>>;
    async fn is_break_time(&self) -> bool;
}

#[async_trait]
pub trait ActivityAnalyzer: Send + Sync {
    async fn analyze_productivity(&self, activities: &[Activity]) -> AppResult<ProductivityStats>;
    async fn analyze_categories(&self, activities: &[Activity]) -> AppResult<Vec<CategoryStats>>;
    async fn get_daily_distribution(&self, activities: &[Activity]) -> AppResult<Vec<(u32, Duration)>>;
}

#[async_trait]
pub trait DataExporter: Send + Sync {
    async fn export_activities_csv(&self, activities: &[Activity]) -> AppResult<String>;
    async fn export_pomodoros_csv(&self, sessions: &[PomodoroSession]) -> AppResult<String>;
    async fn export_json(&self, activities: &[Activity], sessions: &[PomodoroSession]) -> AppResult<String>;
}

#[async_trait]
pub trait NotificationManager: Send + Sync {
    async fn notify(&self, title: &str, message: &str) -> AppResult<()>;
    async fn notify_with_action(&self, title: &str, message: &str, action: &str) -> AppResult<()>;
    async fn clear_notifications(&self) -> AppResult<()>;
}

#[async_trait]
pub trait ConfigManager: Send + Sync {
    async fn load_config<T: serde::de::DeserializeOwned>(&self) -> AppResult<T>;
    async fn save_config<T: serde::Serialize>(&self, config: &T) -> AppResult<()>;
    async fn get_config_path(&self) -> AppResult<std::path::PathBuf>;
}

#[async_trait]
pub trait Cache: Send + Sync {
    async fn get<K: Send + Sync, V: Send + Sync>(&self, key: &K) -> AppResult<Option<V>>;
    async fn set<K: Send + Sync, V: Send + Sync>(&self, key: K, value: V, ttl: Option<Duration>) -> AppResult<()>;
    async fn remove<K: Send + Sync>(&self, key: &K) -> AppResult<()>;
    async fn clear(&self) -> AppResult<()>;
} 