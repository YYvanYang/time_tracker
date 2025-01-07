use crate::core::AppResult;
use crate::plugins::traits::{Plugin, BackupPlugin};
use async_trait::async_trait;
use chrono::Local;
use std::any::Any;
use std::path::{Path, PathBuf};
use std::fs;
use tokio::fs as async_fs;

pub struct FileBackupPlugin {
    backup_dir: PathBuf,
}

impl FileBackupPlugin {
    pub fn new<P: AsRef<Path>>(backup_dir: P) -> Self {
        Self {
            backup_dir: backup_dir.as_ref().to_path_buf(),
        }
    }

    fn get_backup_path(&self, backup_id: &str) -> PathBuf {
        self.backup_dir.join(format!("{}.backup", backup_id))
    }
}

#[async_trait]
impl Plugin for FileBackupPlugin {
    fn name(&self) -> &str {
        "file_backup"
    }

    fn version(&self) -> &str {
        "1.0.0"
    }

    fn description(&self) -> &str {
        "文件备份插件"
    }

    async fn initialize(&self) -> AppResult<()> {
        // 确保备份目录存在
        if !self.backup_dir.exists() {
            async_fs::create_dir_all(&self.backup_dir).await?;
        }
        Ok(())
    }

    async fn start(&self) -> AppResult<()> {
        Ok(())
    }

    async fn stop(&self) -> AppResult<()> {
        Ok(())
    }

    async fn uninstall(&self) -> AppResult<()> {
        // 清理备份目录
        if self.backup_dir.exists() {
            async_fs::remove_dir_all(&self.backup_dir).await?;
        }
        Ok(())
    }

    fn get_settings_ui(&self) -> Option<Box<dyn Any>> {
        None
    }
}

#[async_trait]
impl BackupPlugin for FileBackupPlugin {
    async fn create_backup(&self) -> AppResult<()> {
        let timestamp = Local::now().format("%Y%m%d_%H%M%S");
        let backup_id = format!("backup_{}", timestamp);
        let backup_path = self.get_backup_path(&backup_id);

        // TODO: 实现实际的备份逻辑
        // 1. 导出数据库
        // 2. 导出配置文件
        // 3. 压缩备份文件

        Ok(())
    }

    async fn restore_backup(&self, backup_id: &str) -> AppResult<()> {
        let backup_path = self.get_backup_path(backup_id);
        if !backup_path.exists() {
            return Err(crate::core::AppError::NotFound(format!(
                "备份 {} 不存在",
                backup_id
            )));
        }

        // TODO: 实现实际的恢复逻辑
        // 1. 验证备份文件完整性
        // 2. 停止所有活动的任务
        // 3. 恢复数据库
        // 4. 恢复配置文件

        Ok(())
    }

    async fn list_backups(&self) -> AppResult<Vec<String>> {
        let mut backups = Vec::new();
        let mut entries = async_fs::read_dir(&self.backup_dir).await?;

        while let Some(entry) = entries.next_entry().await? {
            if let Some(filename) = entry.file_name().to_str() {
                if filename.ends_with(".backup") {
                    backups.push(filename[..filename.len() - 7].to_string());
                }
            }
        }

        backups.sort_by(|a, b| b.cmp(a)); // 按时间倒序排序
        Ok(backups)
    }

    async fn delete_backup(&self, backup_id: &str) -> AppResult<()> {
        let backup_path = self.get_backup_path(backup_id);
        if backup_path.exists() {
            async_fs::remove_file(backup_path).await?;
        }
        Ok(())
    }
} 