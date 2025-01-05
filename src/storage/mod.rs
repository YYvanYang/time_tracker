// src/storage/mod.rs

mod models;
mod migrations;
pub mod queries;

use crate::error::{Result, TimeTrackerError};
use chrono::{DateTime, Local, NaiveDateTime};
use rusqlite::{Connection, params};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

pub use models::*;

pub struct Storage {
    conn: Arc<Mutex<Connection>>,
    config: StorageConfig,
}

#[derive(Debug, Clone)]
pub struct StorageConfig {
    pub data_dir: PathBuf,
    pub backup_enabled: bool,
    pub backup_interval: std::time::Duration,
    pub max_backup_count: u32,
    pub vacuum_threshold: u64,  // 数据库大小超过此值时自动清理（字节）
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
        std::fs::create_dir_all(&config.data_dir)?;
        let db_path = config.data_dir.join("timetracker.db");
        
        let conn = Connection::open(&db_path)?;
        
        // 启用外键约束
        conn.execute("PRAGMA foreign_keys = ON", [])?;
        
        // 运行迁移
        migrations::run_migrations(&conn)?;
        
        Ok(Self {
            conn: Arc::new(Mutex::new(conn)),
            config,
        })
    }

    /// 创建数据库备份
    pub fn create_backup(&self) -> Result<PathBuf> {
        let conn = self.conn.lock().map_err(|_| {
            TimeTrackerError::Storage("Failed to lock connection".into())
        })?;

        let backup_dir = self.config.data_dir.join("backups");
        std::fs::create_dir_all(&backup_dir)?;

        let timestamp = Local::now().format("%Y%m%d_%H%M%S");
        let backup_path = backup_dir.join(format!("timetracker_{}.db", timestamp));

        // 创建备份
        let backup_conn = Connection::open(&backup_path)?;
        conn.backup(rusqlite::DatabaseName::Main, &backup_conn, None)?;

        // 清理旧备份
        self.cleanup_old_backups()?;

        Ok(backup_path)
    }

    /// 清理旧的备份文件
    fn cleanup_old_backups(&self) -> Result<()> {
        let backup_dir = self.config.data_dir.join("backups");
        if !backup_dir.exists() {
            return Ok(());
        }

        let mut backups: Vec<_> = std::fs::read_dir(&backup_dir)?
            .filter_map(Result::ok)
            .filter(|entry| {
                entry.path()
                    .extension()
                    .and_then(|ext| ext.to_str())
                    .map_or(false, |ext| ext == "db")
            })
            .collect();

        // 按修改时间排序,使用稳定排序
        backups.sort_by_key(|entry| {
            entry.metadata()
                .and_then(|meta| meta.modified())
                .unwrap_or_else(|_| std::time::SystemTime::UNIX_EPOCH)
        });

        // 使用chunk_exact优化删除逻辑
        if backups.len() > self.config.max_backup_count as usize {
            for backup in backups.iter().take(backups.len() - self.config.max_backup_count as usize) {
                if let Err(e) = std::fs::remove_file(backup.path()) {
                    log::warn!("Failed to remove old backup: {}", e);
                }
            }
        }

        Ok(())
    }

    /// 压缩数据库
    pub fn vacuum(&self) -> Result<()> {
        let conn = self.conn.lock().map_err(|_| {
            TimeTrackerError::Storage("Failed to lock connection".into())
        })?;

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
        let conn = self.conn.lock().map_err(|_| {
            TimeTrackerError::Storage("Failed to lock connection".into())
        })?;

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

    /// 开始事务
    pub fn transaction<T, F>(&self, f: F) -> Result<T>
    where
        F: FnOnce(&Connection) -> Result<T>,
    {
        let conn = self.conn.lock().map_err(|_| {
            TimeTrackerError::Storage("Failed to lock connection".into())
        })?;

        let tx = conn.transaction()?;
        let result = f(&tx)?;
        tx.commit()?;
        
        Ok(result)
    }

    /// 检查数据库健康状况
    pub fn check_health(&self) -> Result<StorageHealth> {
        let conn = self.conn.lock().map_err(|_| {
            TimeTrackerError::Storage("Failed to lock connection".into())
        })?;

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

    // 添加显式的关闭方法
    pub fn close(self) -> Result<()> {
        let conn = Arc::try_unwrap(self.conn)
            .map_err(|_| TimeTrackerError::Storage("Connection still in use".into()))?;
        let conn = conn.into_inner()
            .map_err(|_| TimeTrackerError::Storage("Failed to unwrap connection".into()))?;
        conn.close().map_err(|e| TimeTrackerError::Storage(e.to_string()))?;
        Ok(())
    }

    pub fn backup(&self) -> Result<()> {
        let mut conn = self.conn.lock();
        
        let backup_dir = self.config.data_dir.join("backups");
        std::fs::create_dir_all(&backup_dir)?;

        let backup_path = backup_dir.join(format!(
            "backup_{}.db",
            Local::now().format("%Y%m%d_%H%M%S")
        ));

        let backup_conn = rusqlite::Connection::open(&backup_path)?;
        rusqlite::backup::Backup::new(&*conn, &backup_conn)?.step(-1)?;

        Ok(())
    }

    pub fn list_backups(&self) -> Result<Vec<PathBuf>> {
        let backup_dir = self.config.data_dir.join("backups");
        if !backup_dir.exists() {
            return Ok(Vec::new());
        }

        let entries = std::fs::read_dir(&backup_dir)?
            .filter_map(|e| e.ok())
            .filter(|e| {
                e.path()
                    .extension()
                    .and_then(|ext| ext.to_str())
                    .map(|ext| ext == "db")
                    .unwrap_or(false)
            })
            .map(|e| e.path())
            .collect();

        Ok(entries)
    }

    pub fn execute_transaction<F, T>(&self, f: F) -> Result<T>
    where
        F: FnOnce(&rusqlite::Transaction) -> Result<T>,
    {
        let mut conn = self.conn.lock();
        let tx = conn.transaction()?;
        let result = f(&tx)?;
        tx.commit()?;
        Ok(result)
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

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn create_test_storage() -> (Storage, TempDir) {
        let temp_dir = TempDir::new().unwrap();
        let config = StorageConfig {
            data_dir: temp_dir.path().to_path_buf(),
            ..StorageConfig::default()
        };
        let storage = Storage::new(config).unwrap();
        (storage, temp_dir)
    }

    #[test]
    fn test_storage_initialization() {
        let (storage, _temp_dir) = create_test_storage();
        let health = storage.check_health().unwrap();
        assert!(health.is_healthy);
    }

    #[test]
    fn test_backup_and_cleanup() -> Result<()> {
        let (storage, _temp_dir) = create_test_storage();
        
        // 创建多个备份
        for _ in 0..10 {
            storage.create_backup()?;
        }

        // 验证备份数量限制
        let backup_dir = storage.config.data_dir.join("backups");
        let backup_count = std::fs::read_dir(backup_dir)?
            .filter_map(|entry| entry.ok())
            .count();

        assert_eq!(backup_count, storage.config.max_backup_count as usize);

        Ok(())
    }

    #[test]
    fn test_cleanup_old_data() -> Result<()> {
        let (storage, _temp_dir) = create_test_storage();
        
        // 清理30天前的数据
        storage.cleanup_old_data(30)?;
        
        // 验证清理后的数据
        let conn = storage.conn.lock().unwrap();
        let count: i64 = conn.query_row(
            "SELECT COUNT(*) FROM app_usage WHERE start_time < ?",
            params![Local::now() - chrono::Duration::days(30)],
            |row| row.get(0)
        )?;

        assert_eq!(count, 0);
        Ok(())
    }
}