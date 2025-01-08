use async_trait::async_trait;
use crate::core::{AppResult, models::*};

#[async_trait]
pub trait Storage: Send + Sync {
    // 活动相关
    async fn save_activity(&self, activity: &Activity) -> AppResult<Activity>;
    async fn update_activity(&self, activity: &Activity) -> AppResult<()>;
    async fn get_activity(&self, id: i64) -> AppResult<Activity>;
    async fn get_activities(&self, start: chrono::DateTime<chrono::Local>, end: chrono::DateTime<chrono::Local>) -> AppResult<Vec<Activity>>;
    async fn delete_activity(&self, id: i64) -> AppResult<()>;

    // 项目相关
    async fn save_project(&self, project: &Project) -> AppResult<Project>;
    async fn update_project(&self, project: &Project) -> AppResult<()>;
    async fn get_project(&self, id: i64) -> AppResult<Project>;
    async fn get_projects(&self) -> AppResult<Vec<Project>>;
    async fn delete_project(&self, id: i64) -> AppResult<()>;

    // 番茄钟相关
    async fn save_pomodoro(&self, pomodoro: &PomodoroSession) -> AppResult<PomodoroSession>;
    async fn update_pomodoro(&self, pomodoro: &PomodoroSession) -> AppResult<()>;
    async fn get_pomodoro(&self, id: i64) -> AppResult<PomodoroSession>;
    async fn get_pomodoros(&self, start: chrono::DateTime<chrono::Local>, end: chrono::DateTime<chrono::Local>) -> AppResult<Vec<PomodoroSession>>;
    async fn delete_pomodoro(&self, id: i64) -> AppResult<()>;

    // 标签相关
    async fn save_tag(&self, tag: &Tag) -> AppResult<Tag>;
    async fn update_tag(&self, tag: &Tag) -> AppResult<()>;
    async fn get_tag(&self, id: i64) -> AppResult<Tag>;
    async fn get_tags(&self) -> AppResult<Vec<Tag>>;
    async fn delete_tag(&self, id: i64) -> AppResult<()>;
} 