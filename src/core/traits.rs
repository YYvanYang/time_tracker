use async_trait::async_trait;
use crate::core::models::*;
use crate::core::error::AppResult;
use chrono::{DateTime, Local};
use crate::domain::config::AppConfig;

#[async_trait]
pub trait Storage: Send + Sync {
    async fn initialize(&self) -> AppResult<()>;
    
    // 配置相关
    async fn get_config(&self) -> AppResult<Option<AppConfig>>;
    async fn save_config(&self, config: &AppConfig) -> AppResult<()>;
    
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

#[async_trait]
pub trait ActivityService: TimeTracker {
    async fn get_activities(&self, start: DateTime<Local>, end: DateTime<Local>) -> AppResult<Vec<Activity>>;
    async fn get_project_activities(&self, project_id: i64, start: DateTime<Local>, end: DateTime<Local>) -> AppResult<Vec<Activity>>;
}

#[async_trait]
pub trait ProjectService: Send + Sync {
    async fn create_project(&self, project: Project) -> AppResult<i64>;
    async fn update_project(&self, project: Project) -> AppResult<()>;
    async fn delete_project(&self, id: i64) -> AppResult<()>;
    async fn get_project(&self, id: i64) -> AppResult<Project>;
    async fn list_projects(&self) -> AppResult<Vec<Project>>;
}

#[async_trait]
pub trait PomodoroService: PomodoroTimer {
    async fn get_sessions(&self, start: DateTime<Local>, end: DateTime<Local>) -> AppResult<Vec<PomodoroSession>>;
    async fn get_project_sessions(&self, project_id: i64, start: DateTime<Local>, end: DateTime<Local>) -> AppResult<Vec<PomodoroSession>>;
}

#[async_trait]
pub trait AnalysisService: Send + Sync {
    async fn get_daily_summary(&self, date: DateTime<Local>) -> AppResult<DailySummary>;
    async fn get_weekly_summary(&self, start: DateTime<Local>) -> AppResult<WeeklySummary>;
    async fn get_monthly_summary(&self, start: DateTime<Local>) -> AppResult<MonthlySummary>;
}

#[async_trait]
pub trait ExportService: Send + Sync {
    async fn export_activities(&self, start: DateTime<Local>, end: DateTime<Local>, format: ExportFormat) -> AppResult<Vec<u8>>;
    async fn export_pomodoros(&self, start: DateTime<Local>, end: DateTime<Local>, format: ExportFormat) -> AppResult<Vec<u8>>;
} 