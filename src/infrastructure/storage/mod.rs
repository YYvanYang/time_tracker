mod models;
mod queries;

pub use models::*;
pub use queries::*;

use crate::core::{AppError, AppResult};
use crate::domain::config::AppConfig;
use crate::core::models::{Activity, Project, PomodoroSession};
use sqlx::{
    sqlite::{SqlitePool, SqlitePoolOptions},
    Pool, Sqlite, Row,
};
use std::path::Path;
use tokio::sync::OnceCell;
use async_trait::async_trait;
use crate::core::traits::Storage;
use chrono::{DateTime, Local};

pub struct SqliteStorage {
    pool: Pool<Sqlite>,
}

impl SqliteStorage {
    pub async fn new(database_path: impl AsRef<Path>) -> AppResult<Self> {
        let pool = SqlitePoolOptions::new()
            .max_connections(5)
            .connect_with(
                sqlx::sqlite::SqliteConnectOptions::new()
                    .filename(database_path.as_ref())
                    .create_if_missing(true)
                    .journal_mode(sqlx::sqlite::SqliteJournalMode::Wal)
                    .foreign_keys(true),
            )
            .await?;

        // 运行迁移
        sqlx::migrate!("./migrations").run(&pool).await?;

        Ok(Self { pool })
    }

    pub async fn backup(&self, backup_path: impl AsRef<Path>) -> AppResult<()> {
        let backup_path = backup_path.as_ref().to_string_lossy();
        sqlx::query(&format!("VACUUM INTO '{}'", backup_path))
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    pub async fn vacuum(&self) -> AppResult<()> {
        sqlx::query("VACUUM")
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    pub async fn transaction<F, T>(&self, f: F) -> AppResult<T>
    where
        F: FnOnce(&mut sqlx::Transaction<'_, Sqlite>) -> AppResult<T>,
    {
        let mut tx = self.pool.begin().await?;
        let result = f(&mut tx).await?;
        tx.commit().await?;
        Ok(result)
    }
}

#[async_trait]
impl Storage for SqliteStorage {
    async fn initialize(&self) -> AppResult<()> {
        Ok(())
    }

    async fn get_config(&self) -> AppResult<Option<AppConfig>> {
        let result = sqlx::query("SELECT * FROM config WHERE id = 1")
            .fetch_optional(&self.pool)
            .await?;

        match result {
            Some(row) => {
                let data: String = row.get("data");
                let config: AppConfig = serde_json::from_str(&data)?;
                Ok(Some(config))
            }
            None => Ok(None),
        }
    }

    async fn save_config(&self, config: &AppConfig) -> AppResult<()> {
        let data = serde_json::to_string(config)?;
        sqlx::query(
            r#"
            INSERT INTO config (id, data) VALUES (1, ?)
            ON CONFLICT(id) DO UPDATE SET data = excluded.data
            "#,
        )
        .bind(&data)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    async fn save_activity(&self, activity: &Activity) -> AppResult<i64> {
        let result = sqlx::query(
            r#"
            INSERT INTO activities (
                title, description, start_time, end_time, project_id, category_id
            ) VALUES (?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(&activity.title)
        .bind(&activity.description)
        .bind(&activity.start_time)
        .bind(&activity.end_time)
        .bind(&activity.project_id)
        .bind(&activity.category_id)
        .execute(&self.pool)
        .await?;
        Ok(result.last_insert_rowid())
    }

    async fn get_activity(&self, id: i64) -> AppResult<Activity> {
        let activity = sqlx::query_as::<_, Activity>(
            r#"
            SELECT * FROM activities WHERE id = ?
            "#,
        )
        .bind(id)
        .fetch_one(&self.pool)
        .await?;
        Ok(activity)
    }

    async fn list_activities(&self) -> AppResult<Vec<Activity>> {
        let activities = sqlx::query_as::<_, Activity>(
            r#"
            SELECT * FROM activities ORDER BY start_time DESC
            "#,
        )
        .fetch_all(&self.pool)
        .await?;
        Ok(activities)
    }

    async fn get_activities(&self, start: DateTime<Local>, end: DateTime<Local>) -> AppResult<Vec<Activity>> {
        let activities = sqlx::query_as::<_, Activity>(
            r#"
            SELECT * FROM activities 
            WHERE start_time >= ? AND end_time <= ?
            ORDER BY start_time DESC
            "#,
        )
        .bind(start)
        .bind(end)
        .fetch_all(&self.pool)
        .await?;
        Ok(activities)
    }

    async fn get_project_activities(&self, project_id: i64, start: DateTime<Local>, end: DateTime<Local>) -> AppResult<Vec<Activity>> {
        let activities = sqlx::query_as::<_, Activity>(
            r#"
            SELECT * FROM activities 
            WHERE project_id = ? AND start_time >= ? AND end_time <= ?
            ORDER BY start_time DESC
            "#,
        )
        .bind(project_id)
        .bind(start)
        .bind(end)
        .fetch_all(&self.pool)
        .await?;
        Ok(activities)
    }

    async fn save_project(&self, project: &Project) -> AppResult<i64> {
        let result = sqlx::query(
            r#"
            INSERT INTO projects (
                name, description, color, created_at, updated_at
            ) VALUES (?, ?, ?, ?, ?)
            "#,
        )
        .bind(&project.name)
        .bind(&project.description)
        .bind(&project.color)
        .bind(&project.created_at)
        .bind(&project.updated_at)
        .execute(&self.pool)
        .await?;
        Ok(result.last_insert_rowid())
    }

    async fn get_project(&self, id: i64) -> AppResult<Project> {
        let project = sqlx::query_as::<_, Project>(
            r#"
            SELECT * FROM projects WHERE id = ?
            "#,
        )
        .bind(id)
        .fetch_one(&self.pool)
        .await?;
        Ok(project)
    }

    async fn list_projects(&self) -> AppResult<Vec<Project>> {
        let projects = sqlx::query_as::<_, Project>(
            r#"
            SELECT * FROM projects ORDER BY name
            "#,
        )
        .fetch_all(&self.pool)
        .await?;
        Ok(projects)
    }

    async fn save_pomodoro(&self, pomodoro: &PomodoroSession) -> AppResult<i64> {
        let result = sqlx::query(
            r#"
            INSERT INTO pomodoro_sessions (
                start_time, end_time, duration, status, project_id
            ) VALUES (?, ?, ?, ?, ?)
            "#,
        )
        .bind(&pomodoro.start_time)
        .bind(&pomodoro.end_time)
        .bind(&pomodoro.duration)
        .bind(&pomodoro.status)
        .bind(&pomodoro.project_id)
        .execute(&self.pool)
        .await?;
        Ok(result.last_insert_rowid())
    }

    async fn get_pomodoro(&self, id: i64) -> AppResult<PomodoroSession> {
        let session = sqlx::query_as::<_, PomodoroSession>(
            r#"
            SELECT * FROM pomodoro_sessions WHERE id = ?
            "#,
        )
        .bind(id)
        .fetch_one(&self.pool)
        .await?;
        Ok(session)
    }

    async fn list_pomodoros(&self) -> AppResult<Vec<PomodoroSession>> {
        let sessions = sqlx::query_as::<_, PomodoroSession>(
            r#"
            SELECT * FROM pomodoro_sessions ORDER BY start_time DESC
            "#,
        )
        .fetch_all(&self.pool)
        .await?;
        Ok(sessions)
    }

    async fn get_pomodoro_sessions(&self, start: DateTime<Local>, end: DateTime<Local>) -> AppResult<Vec<PomodoroSession>> {
        let sessions = sqlx::query_as::<_, PomodoroSession>(
            r#"
            SELECT * FROM pomodoro_sessions 
            WHERE start_time >= ? AND end_time <= ?
            ORDER BY start_time DESC
            "#,
        )
        .bind(start)
        .bind(end)
        .fetch_all(&self.pool)
        .await?;
        Ok(sessions)
    }

    async fn get_project_pomodoro_sessions(&self, project_id: i64, start: DateTime<Local>, end: DateTime<Local>) -> AppResult<Vec<PomodoroSession>> {
        let sessions = sqlx::query_as::<_, PomodoroSession>(
            r#"
            SELECT * FROM pomodoro_sessions 
            WHERE project_id = ? AND start_time >= ? AND end_time <= ?
            ORDER BY start_time DESC
            "#,
        )
        .bind(project_id)
        .bind(start)
        .bind(end)
        .fetch_all(&self.pool)
        .await?;
        Ok(sessions)
    }
}

#[derive(sqlx::FromRow)]
struct ConfigRow {
    id: i64,
    data: String,
}