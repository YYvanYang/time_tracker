mod models;
mod queries;

pub use models::*;
pub use queries::*;

use crate::core::{AppError, AppResult};
use sqlx::{
    sqlite::{SqlitePool, SqlitePoolOptions},
    Pool, Sqlite,
};
use std::path::Path;
use tokio::sync::OnceCell;

pub struct Storage {
    pool: Pool<Sqlite>,
}

impl Storage {
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

    pub async fn get_app_usage_records(
        &self,
        start: chrono::DateTime<chrono::Local>,
        end: chrono::DateTime<chrono::Local>,
    ) -> AppResult<Vec<AppUsageRecord>> {
        queries::get_app_usage_records(&self.pool, start, end).await
    }

    pub async fn get_pomodoro_records(
        &self,
        start: chrono::DateTime<chrono::Local>,
        end: chrono::DateTime<chrono::Local>,
    ) -> AppResult<Vec<PomodoroRecord>> {
        queries::get_pomodoro_records(&self.pool, start, end).await
    }

    pub async fn insert_app_usage(&self, record: &AppUsageRecord) -> AppResult<i64> {
        queries::insert_app_usage(&self.pool, record).await
    }

    pub async fn insert_pomodoro(&self, record: &PomodoroRecord) -> AppResult<i64> {
        queries::insert_pomodoro(&self.pool, record).await
    }

    pub async fn get_statistics(
        &self,
        start: chrono::DateTime<chrono::Local>,
        end: chrono::DateTime<chrono::Local>,
    ) -> AppResult<Vec<(String, String)>> {
        queries::get_statistics(&self.pool, start, end).await
    }
}

static INSTANCE: OnceCell<Storage> = OnceCell::const_new();

impl Storage {
    pub async fn initialize(database_path: impl AsRef<Path>) -> AppResult<()> {
        let storage = Self::new(database_path).await?;
        INSTANCE.set(storage).map_err(|_| {
            AppError::Storage("Storage has already been initialized".into())
        })?;
        Ok(())
    }

    pub fn instance() -> AppResult<&'static Storage> {
        INSTANCE.get().ok_or_else(|| {
            AppError::Storage("Storage has not been initialized".into())
        })
    }
}