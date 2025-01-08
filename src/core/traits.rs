use async_trait::async_trait;
use crate::core::models::*;
use crate::core::error::AppResult;
use chrono::{DateTime, Local};

#[async_trait]
pub trait Storage: Send + Sync {
    async fn initialize(&self) -> AppResult<()>;
    
    // 活动相关
    async fn save_activity(&self, activity: &Activity) -> AppResult<i64>;
    async fn get_activity(&self, id: i64) -> AppResult<Activity>;
    async fn list_activities(&self) -> AppResult<Vec<Activity>>;
    async fn get_activities(&self, start: DateTime<Local>, end: DateTime<Local>) -> AppResult<Vec<Activity>>;
    async fn get_project_activities(&self, project_id: i64, start: DateTime<Local>, end: DateTime<Local>) -> AppResult<Vec<Activity>>;
    
    // 项目相关
    async fn save_project(&self, project: &Project) -> AppResult<i64>;
    async fn get_project(&self, id: i64) -> AppResult<Project>;
    async fn list_projects(&self) -> AppResult<Vec<Project>>;
    
    // 番茄钟相关
    async fn save_pomodoro(&self, pomodoro: &PomodoroSession) -> AppResult<i64>;
    async fn get_pomodoro(&self, id: i64) -> AppResult<PomodoroSession>;
    async fn list_pomodoros(&self) -> AppResult<Vec<PomodoroSession>>;
    async fn get_pomodoro_sessions(&self, start: DateTime<Local>, end: DateTime<Local>) -> AppResult<Vec<PomodoroSession>>;
    async fn get_project_pomodoro_sessions(&self, project_id: i64, start: DateTime<Local>, end: DateTime<Local>) -> AppResult<Vec<PomodoroSession>>;
}

#[async_trait]
pub trait TimeTracker {
    async fn start_tracking(&self, activity: Activity) -> AppResult<()>;
    async fn stop_tracking(&self) -> AppResult<()>;
    async fn get_current_activity(&self) -> AppResult<Option<Activity>>;
    async fn is_tracking(&self) -> AppResult<bool>;
}

#[async_trait]
pub trait PomodoroTimer {
    async fn start_session(&self, duration: i32) -> AppResult<()>;
    async fn pause_session(&self) -> AppResult<()>;
    async fn resume_session(&self) -> AppResult<()>;
    async fn stop_session(&self) -> AppResult<()>;
    async fn get_current_session(&self) -> AppResult<Option<PomodoroSession>>;
    async fn is_active(&self) -> AppResult<bool>;
} 