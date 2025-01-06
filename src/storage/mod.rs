// src/storage/mod.rs

mod models;
mod migrations;
pub mod queries;
pub mod app_state;

use crate::error::{Result, TimeTrackerError};
use chrono::{DateTime, Local, NaiveDateTime};
use rusqlite::{Connection, params};
use std::path::PathBuf;
use std::sync::Arc;
use r2d2_sqlite::SqliteConnectionManager;
use r2d2::Pool;
use crate::config;
use serde::{Serialize, Deserialize};
use std::time::Duration;

pub use models::*;
pub use app_state::AppStateManager;

pub struct Storage {
    pool: Pool<SqliteConnectionManager>,
    config: StorageConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageConfig {
    pub data_dir: PathBuf,
    pub backup_enabled: bool,
    pub backup_interval: Duration,
    #[serde(default = "default_max_backup_count")]
    pub max_backup_count: u32,
    #[serde(default = "default_vacuum_threshold")]
    pub vacuum_threshold: u64,
}

fn default_max_backup_count() -> u32 {
    7
}

fn default_vacuum_threshold() -> u64 {
    100 * 1024 * 1024  // 100MB
}

impl Default for StorageConfig {
    fn default() -> Self {
        Self {
            data_dir: dirs::data_dir()
                .unwrap_or_else(|| PathBuf::from("."))
                .join("time_tracker"),
            backup_enabled: true,
            backup_interval: std::time::Duration::from_secs(24 * 60 * 60), // 每天
            max_backup_count: 7,  // 保留最近7个备份
            vacuum_threshold: 100 * 1024 * 1024,  // 100MB
        }
    }
}

impl Storage {
    pub fn new(config: StorageConfig) -> Result<Self> {
        let manager = SqliteConnectionManager::file(&config.data_dir.join("timetracker.db"));
        let pool = Pool::new(manager)?;
        
        // 初始化数据库
        let mut conn = pool.get()?;
        migrations::run_migrations(&mut conn)?;
        
        Ok(Self { pool, config })
    }

    pub fn new_in_memory() -> Result<Self> {
        let manager = SqliteConnectionManager::memory();
        let pool = Pool::new(manager)?;
        let config = StorageConfig::default();
        
        // 初始化数据库
        let mut conn = pool.get()?;
        migrations::run_migrations(&mut conn)?;
        
        Ok(Self { pool, config })
    }

    /// 创建数据库备份
    pub fn backup(&self) -> Result<PathBuf> {
        let mut conn = self.pool.get()?;
        let backup_dir = self.config.data_dir.join("backups");
        std::fs::create_dir_all(&backup_dir)?;

        // 使用更精确的时间戳格式
        let backup_path = backup_dir.join(format!(
            "backup_{}.db",
            Local::now().format("%Y%m%d_%H%M%S_%3f")
        ));

        let mut backup_conn = rusqlite::Connection::open(&backup_path)?;
        
        // 使用 backup API
        let backup = rusqlite::backup::Backup::new(&*conn, &mut backup_conn)?;
        backup.step(-1)?;

        // 清理旧备份
        self.cleanup_old_backups()?;

        Ok(backup_path)
    }

    /// 清理旧的备份文件
    pub fn cleanup_old_backups(&self) -> Result<()> {
        let backup_dir = self.config.data_dir.join("backups");
        if !backup_dir.exists() {
            return Ok(());
        }

        let mut backups: Vec<_> = std::fs::read_dir(&backup_dir)?
            .filter_map(|entry| entry.ok())
            .filter(|entry| {
                entry.path()
                    .extension()
                    .and_then(|ext| ext.to_str())
                    .map_or(false, |ext| ext == "db")
            })
            .collect();

        // 按修改时间排序
        backups.sort_by_key(|entry| {
            entry.metadata()
                .and_then(|meta| meta.modified())
                .unwrap_or_else(|_| std::time::SystemTime::UNIX_EPOCH)
        });

        // 保留最新的 max_backup_count 个备份
        while backups.len() > self.config.max_backup_count as usize {
            if let Some(oldest) = backups.first() {
                std::fs::remove_file(oldest.path())?;
                backups.remove(0);
            }
        }

        Ok(())
    }

    /// 压缩数据库
    pub fn vacuum(&self) -> Result<()> {
        let mut conn = self.pool.get()?;

        // 检查数据库大小
        let db_size: i64 = conn.query_row(
            "SELECT page_count * page_size FROM pragma_page_count, pragma_page_size",
            [],
            |row| row.get(0)
        )?;

        if db_size as u64 > self.config.vacuum_threshold {
            conn.execute("VACUUM", [])?;
        }

        Ok(())
    }

    /// 清理旧数据
    pub fn cleanup_old_data(&self, days: u32) -> Result<()> {
        let conn = self.pool.get()?;
        let cutoff_date = Local::now() - chrono::Duration::days(days as i64);
        
        conn.execute(
            "DELETE FROM app_usage WHERE start_time < ?",
            params![cutoff_date],
        )?;

        conn.execute(
            "DELETE FROM pomodoro_records WHERE start_time < ?",
            params![cutoff_date],
        )?;

        Ok(())
    }

    /// 检查数据库健康状况
    pub fn check_health(&self) -> Result<StorageHealth> {
        let conn = self.pool.get()?;

        // 检查数据库完整性
        let integrity_check: String = conn.query_row(
            "PRAGMA integrity_check",
            [],
            |row| row.get(0)
        )?;

        let is_healthy = integrity_check == "ok";

        // 获取数据库信息
        let page_size: i64 = conn.query_row("PRAGMA page_size", [], |row| row.get(0))?;
        let page_count: i64 = conn.query_row("PRAGMA page_count", [], |row| row.get(0))?;
        let size = page_size * page_count;

        // 获取各表的记录数
        let app_usage_count: i64 = conn.query_row(
            "SELECT COUNT(*) FROM app_usage",
            [],
            |row| row.get(0)
        )?;

        let pomodoro_count: i64 = conn.query_row(
            "SELECT COUNT(*) FROM pomodoro_records",
            [],
            |row| row.get(0)
        )?;

        Ok(StorageHealth {
            is_healthy,
            database_size: size as u64,
            app_usage_count: app_usage_count as u64,
            pomodoro_count: pomodoro_count as u64,
            last_backup: self.get_last_backup_time()?,
            needs_vacuum: size as u64 > self.config.vacuum_threshold,
        })
    }

    fn get_last_backup_time(&self) -> Result<Option<DateTime<Local>>> {
        let backup_dir = self.config.data_dir.join("backups");
        if !backup_dir.exists() {
            return Ok(None);
        }

        let latest_backup = std::fs::read_dir(&backup_dir)?
            .filter_map(|entry| entry.ok())
            .filter(|entry| {
                entry.path().extension()
                    .and_then(|ext| ext.to_str())
                    .map_or(false, |ext| ext == "db")
            })
            .max_by_key(|entry| {
                entry.metadata()
                    .and_then(|meta| meta.modified())
                    .unwrap_or_else(|_| std::time::SystemTime::UNIX_EPOCH)
            });

        if let Some(backup) = latest_backup {
            if let Ok(metadata) = backup.metadata() {
                if let Ok(modified) = metadata.modified() {
                    if let Ok(datetime) = modified.duration_since(std::time::SystemTime::UNIX_EPOCH) {
                        let naive = NaiveDateTime::from_timestamp_opt(
                            datetime.as_secs() as i64,
                            datetime.subsec_nanos()
                        ).unwrap_or_default();
                        return Ok(Some(DateTime::from_naive_utc_and_offset(naive, *Local::now().offset())));
                    }
                }
            }
        }

        Ok(None)
    }

    pub fn list_backups(&self) -> Result<Vec<(PathBuf, DateTime<Local>)>> {
        let backup_dir = self.config.data_dir.join("backups");
        if !backup_dir.exists() {
            return Ok(Vec::new());
        }

        let mut backups = std::fs::read_dir(&backup_dir)?
            .filter_map(|e| e.ok())
            .filter(|e| {
                e.path()
                    .extension()
                    .and_then(|ext| ext.to_str())
                    .map(|ext| ext == "db")
                    .unwrap_or(false)
            })
            .filter_map(|e| {
                let path = e.path();
                let modified = e.metadata().ok()?.modified().ok()?;
                let datetime = DateTime::from(modified);
                Some((path, datetime))
            })
            .collect::<Vec<_>>();

        // 按时间倒序排序
        backups.sort_by(|a, b| b.1.cmp(&a.1));
        Ok(backups)
    }

    pub fn execute_transaction<F, T>(&self, f: F) -> Result<T>
    where
        F: FnOnce(&rusqlite::Transaction) -> Result<T>,
    {
        let mut conn = self.pool.get()?;
        let tx = conn.transaction()?;
        
        match f(&tx) {
            Ok(result) => {
                tx.commit()?;
                Ok(result)
            }
            Err(e) => {
                tx.rollback()?;
                Err(e)
            }
        }
    }

    pub fn execute_query(&self, query: &str) -> Result<()> {
        let conn = self.pool.get()?;
        conn.execute(query, [])?;
        Ok(())
    }

    pub fn add_project(&mut self, project: &Project) -> Result<i64> {
        let conn = self.pool.get()?;
        conn.execute(
            "INSERT INTO projects (name, description, created_at, updated_at) 
             VALUES (?1, ?2, ?3, ?4)",
            params![
                project.name,
                project.description,
                project.created_at,
                project.updated_at,
            ],
        )?;
        
        Ok(conn.last_insert_rowid())
    }

    pub fn delete_project(&mut self, project_id: i64) -> Result<()> {
        let conn = self.pool.get()?;
        conn.execute("DELETE FROM projects WHERE id = ?1", params![project_id])?;
        Ok(())
    }

    pub fn load_projects(&self) -> Result<Vec<Project>> {
        let conn = self.pool.get()?;
        let mut stmt = conn.prepare(
            "SELECT id, name, description, created_at, updated_at FROM projects"
        )?;
        
        let projects = stmt.query_map([], |row| {
            Ok(Project {
                id: row.get(0)?,
                name: row.get(1)?,
                description: row.get(2)?,
                created_at: row.get(3)?,
                updated_at: row.get(4)?,
            })
        })?;

        let mut result = Vec::new();
        for project in projects {
            result.push(project?);
        }
        Ok(result)
    }

    pub fn update_project(&mut self, project: &Project) -> Result<()> {
        let conn = self.pool.get()?;
        conn.execute(
            "UPDATE projects 
             SET name = ?1, description = ?2, updated_at = ?3 
             WHERE id = ?4",
            params![
                project.name,
                project.description,
                project.updated_at,
                project.id,
            ],
        )?;
        Ok(())
    }
}

#[derive(Debug)]
pub struct StorageHealth {
    pub is_healthy: bool,
    pub database_size: u64,
    pub app_usage_count: u64,
    pub pomodoro_count: u64,
    pub last_backup: Option<DateTime<Local>>,
    pub needs_vacuum: bool,
}

impl From<config::StorageConfig> for StorageConfig {
    fn from(config: config::StorageConfig) -> Self {
        Self {
            data_dir: config.data_dir,
            backup_enabled: config.backup_enabled,
            backup_interval: config.backup_interval,
            max_backup_count: config.max_backup_count,
            vacuum_threshold: config.vacuum_threshold,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn create_test_storage() -> Result<(Storage, TempDir)> {
        let temp_dir = TempDir::new()?;
        let config = StorageConfig {
            data_dir: temp_dir.path().to_path_buf(),
            backup_enabled: true,
            backup_interval: Duration::from_secs(60),
            max_backup_count: 3,
            vacuum_threshold: 1024 * 1024,  // 1MB
        };
        let storage = Storage::new(config)?;
        Ok((storage, temp_dir))
    }

    #[test]
    fn test_storage_initialization() {
        let (storage, _temp_dir) = create_test_storage().unwrap();
        let health = storage.check_health().unwrap();
        assert!(health.is_healthy);
    }

    #[test]
    fn test_backup_and_cleanup() -> Result<()> {
        let (storage, _temp_dir) = create_test_storage().unwrap();
        
        // 创建多个备份
        for _ in 0..5 {
            storage.backup()?;
            std::thread::sleep(Duration::from_millis(100)); // 确保时间戳不同
        }

        // 验证备份数量限制
        let backups = storage.list_backups()?;
        assert_eq!(backups.len(), storage.config.max_backup_count as usize);

        // 验证是保留了最新的备份
        let backup_times: Vec<_> = backups.iter()
            .filter_map(|(path, _)| path.metadata().ok())
            .filter_map(|meta| meta.modified().ok())
            .collect();
        
        assert!(backup_times.windows(2).all(|w| w[0] > w[1]));

        Ok(())
    }

    #[test]
    fn test_cleanup_old_data() -> Result<()> {
        let (storage, _temp_dir) = create_test_storage().unwrap();
        
        // 清理30天前的数据
        storage.cleanup_old_data(30)?;
        
        // 验证清理后的数据
        let conn = storage.pool.get()?;
        let count: i64 = conn.query_row(
            "SELECT COUNT(*) FROM app_usage WHERE start_time < ?",
            params![Local::now() - chrono::Duration::days(30)],
            |row| row.get(0)
        )?;

        assert_eq!(count, 0);
        Ok(())
    }

    #[test]
    fn test_transaction() -> Result<()> {
        let (storage, _temp_dir) = create_test_storage().unwrap();

        // 测试成功的事务
        let result: Result<()> = storage.execute_transaction(|tx| {
            tx.execute("CREATE TABLE test (id INTEGER PRIMARY KEY)", [])?;
            tx.execute("INSERT INTO test VALUES (?1)", [1])?;
            Ok(())
        });
        assert!(result.is_ok());

        // 测试失败的事务
        let result: Result<()> = storage.execute_transaction(|tx| {
            tx.execute("INSERT INTO test VALUES (?1)", [2])?;
            Err(TimeTrackerError::Storage("Test rollback".into()))
        });
        assert!(result.is_err());

        // 验证回滚
        let conn = storage.pool.get()?;
        let count: i64 = conn.query_row("SELECT COUNT(*) FROM test", [], |row| row.get(0))?;
        assert_eq!(count, 1);  // 只有第一个事务的数据

        Ok(())
    }
}