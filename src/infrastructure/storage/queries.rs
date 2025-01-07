// src/storage/queries.rs

use chrono::{DateTime, Local};
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, SqlitePool};
use std::time::Duration;

use crate::core::AppResult;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct AppUsageRecord {
    pub id: Option<i64>,
    pub app_name: String,
    pub window_title: String,
    pub start_time: DateTime<Local>,
    pub duration: i64,
    pub category: Option<String>,
    pub productivity_score: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct PomodoroRecord {
    pub id: Option<i64>,
    pub start_time: DateTime<Local>,
    pub end_time: DateTime<Local>,
    pub status: String,
    pub notes: Option<String>,
    pub project_id: Option<i64>,
}

pub async fn get_app_usage_records(
    pool: &SqlitePool,
    start: DateTime<Local>,
    end: DateTime<Local>,
) -> AppResult<Vec<AppUsageRecord>> {
    let records = sqlx::query_as::<_, AppUsageRecord>(
        r#"
        SELECT id, app_name, window_title, start_time, duration, category, productivity_score
        FROM app_usage
        WHERE start_time BETWEEN ? AND ?
        ORDER BY start_time DESC
        "#,
    )
    .bind(start.to_rfc3339())
    .bind(end.to_rfc3339())
    .fetch_all(pool)
    .await?;

    Ok(records)
}

pub async fn get_pomodoro_records(
    pool: &SqlitePool,
    start: DateTime<Local>,
    end: DateTime<Local>,
) -> AppResult<Vec<PomodoroRecord>> {
    let records = sqlx::query_as::<_, PomodoroRecord>(
        r#"
        SELECT id, start_time, end_time, status, notes, project_id
        FROM pomodoro_records
        WHERE start_time BETWEEN ? AND ?
        ORDER BY start_time DESC
        "#,
    )
    .bind(start.to_rfc3339())
    .bind(end.to_rfc3339())
    .fetch_all(pool)
    .await?;

    Ok(records)
}

pub async fn insert_app_usage(
    pool: &SqlitePool,
    record: &AppUsageRecord,
) -> AppResult<i64> {
    let result = sqlx::query(
        r#"
        INSERT INTO app_usage (app_name, window_title, start_time, duration, category, productivity_score)
        VALUES (?, ?, ?, ?, ?, ?)
        "#,
    )
    .bind(&record.app_name)
    .bind(&record.window_title)
    .bind(record.start_time.to_rfc3339())
    .bind(record.duration)
    .bind(&record.category)
    .bind(record.productivity_score)
    .execute(pool)
    .await?;

    Ok(result.last_insert_rowid())
}

pub async fn insert_pomodoro(
    pool: &SqlitePool,
    record: &PomodoroRecord,
) -> AppResult<i64> {
    let result = sqlx::query(
        r#"
        INSERT INTO pomodoro_records (start_time, end_time, status, notes, project_id)
        VALUES (?, ?, ?, ?, ?)
        "#,
    )
    .bind(record.start_time.to_rfc3339())
    .bind(record.end_time.to_rfc3339())
    .bind(&record.status)
    .bind(&record.notes)
    .bind(record.project_id)
    .execute(pool)
    .await?;

    Ok(result.last_insert_rowid())
}

pub async fn get_statistics(
    pool: &SqlitePool,
    start: DateTime<Local>,
    end: DateTime<Local>,
) -> AppResult<Vec<(String, String)>> {
    let mut stats = Vec::new();

    // 总工作时间
    let total_work_time: i64 = sqlx::query_scalar(
        r#"
        SELECT COALESCE(SUM(duration), 0)
        FROM app_usage
        WHERE start_time BETWEEN ? AND ?
        "#,
    )
    .bind(start.to_rfc3339())
    .bind(end.to_rfc3339())
    .fetch_one(pool)
    .await?;

    // 番茄钟统计
    let (completed, interrupted): (i64, i64) = sqlx::query_as(
        r#"
        SELECT 
            COUNT(CASE WHEN status = 'Completed' THEN 1 END) as completed,
            COUNT(CASE WHEN status = 'Interrupted' THEN 1 END) as interrupted
        FROM pomodoro_records
        WHERE start_time BETWEEN ? AND ?
        "#,
    )
    .bind(start.to_rfc3339())
    .bind(end.to_rfc3339())
    .fetch_one(pool)
    .await?;

    stats.push(("总工作时间(小时)".to_string(), format!("{:.1}", total_work_time as f64 / 3600.0)));
    stats.push(("完成的番茄钟".to_string(), completed.to_string()));
    stats.push(("中断的番茄钟".to_string(), interrupted.to_string()));

    Ok(stats)
}