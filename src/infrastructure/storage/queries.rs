// src/storage/queries.rs

use chrono::{DateTime, Local};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppUsageRecord {
    pub id: i64,
    pub app_name: String,
    pub window_title: String,
    #[serde(with = "chrono::serde::ts_seconds")]
    pub start_time: DateTime<Local>,
    pub duration: i64,
    pub category: Option<String>,
    pub project_id: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PomodoroRecord {
    pub id: i64,
    #[serde(with = "chrono::serde::ts_seconds")]
    pub start_time: DateTime<Local>,
    #[serde(with = "chrono::serde::ts_seconds")]
    pub end_time: DateTime<Local>,
    pub status: String,
    pub notes: Option<String>,
    pub project_id: Option<i64>,
}

pub async fn get_app_usage(
    pool: &sqlx::SqlitePool,
    start: DateTime<Local>,
    end: DateTime<Local>,
) -> sqlx::Result<Vec<AppUsageRecord>> {
    let records = sqlx::query!(
        r#"
        SELECT id, app_name, window_title, start_time, duration, category, project_id
        FROM app_usage
        WHERE start_time >= ?
        AND start_time <= ?
        ORDER BY start_time DESC
        "#,
        start.timestamp(),
        end.timestamp()
    )
    .fetch_all(pool)
    .await?;

    Ok(records
        .into_iter()
        .map(|r| AppUsageRecord {
            id: r.id,
            app_name: r.app_name,
            window_title: r.window_title,
            start_time: DateTime::from_timestamp(r.start_time, 0).unwrap_or_default(),
            duration: r.duration,
            category: r.category,
            project_id: r.project_id,
        })
        .collect())
}

pub async fn get_pomodoros(
    pool: &sqlx::SqlitePool,
    start: DateTime<Local>,
    end: DateTime<Local>,
) -> sqlx::Result<Vec<PomodoroRecord>> {
    let records = sqlx::query!(
        r#"
        SELECT id, start_time, end_time, status, notes, project_id
        FROM pomodoro_records
        WHERE start_time >= ?
        AND start_time <= ?
        ORDER BY start_time DESC
        "#,
        start.timestamp(),
        end.timestamp()
    )
    .fetch_all(pool)
    .await?;

    Ok(records
        .into_iter()
        .map(|r| PomodoroRecord {
            id: r.id,
            start_time: DateTime::from_timestamp(r.start_time, 0).unwrap_or_default(),
            end_time: DateTime::from_timestamp(r.end_time, 0).unwrap_or_default(),
            status: r.status,
            notes: r.notes,
            project_id: r.project_id,
        })
        .collect())
}