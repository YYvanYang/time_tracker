use crate::core::{AppResult, models::*};
use crate::domain::plugin::{Plugin, PluginMetadata};
use async_trait::async_trait;
use chrono::{DateTime, Local};
use serde::{Serialize, Deserialize};
use std::path::{Path, PathBuf};
use std::time::Duration;
use tokio::fs;
use tokio::sync::RwLock;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupConfig {
    pub backup_dir: String,
    pub auto_backup: bool,
    pub backup_interval: Duration,
    pub max_backups: usize,
}

impl Default for BackupConfig {
    fn default() -> Self {
        Self {
            backup_dir: "backups".into(),
            auto_backup: true,
            backup_interval: Duration::from_secs(24 * 60 * 60), // 1 day
            max_backups: 10,
        }
    }
}

pub struct BackupPlugin {
    metadata: PluginMetadata,
    config: RwLock<BackupConfig>,
    last_backup: RwLock<Option<DateTime<Local>>>,
}

impl BackupPlugin {
    pub fn new() -> Self {
        Self {
            metadata: PluginMetadata {
                id: "backup".into(),
                name: "Backup Plugin".into(),
                version: "1.0.0".into(),
                author: "Time Tracker".into(),
                description: "Automatically backup application data".into(),
                dependencies: vec![],
                config_schema: Some(serde_json::to_value(BackupConfig::default()).unwrap()),
            },
            config: RwLock::new(BackupConfig::default()),
            last_backup: RwLock::new(None),
        }
    }

    async fn create_backup(&self) -> AppResult<PathBuf> {
        let config = self.config.read().await;
        let backup_dir = Path::new(&config.backup_dir);

        // 确保备份目录存在
        if !backup_dir.exists() {
            fs::create_dir_all(backup_dir).await?;
        }

        // 生成备份文件名
        let timestamp = Local::now().format("%Y%m%d_%H%M%S");
        let backup_file = backup_dir.join(format!("backup_{}.zip", timestamp));

        // 创建 zip 文件
        let file = std::fs::File::create(&backup_file)?;
        let mut zip = zip::ZipWriter::new(file);

        // 添加数据库文件
        let db_path = "time_tracker.db";
        if Path::new(db_path).exists() {
            let db_content = fs::read(db_path).await?;
            zip.start_file("time_tracker.db", zip::write::FileOptions::default())?;
            zip.write_all(&db_content)?;
        }

        // 添加配置文件
        let config_path = "config.json";
        if Path::new(config_path).exists() {
            let config_content = fs::read(config_path).await?;
            zip.start_file("config.json", zip::write::FileOptions::default())?;
            zip.write_all(&config_content)?;
        }

        zip.finish()?;

        // 更新最后备份时间
        *self.last_backup.write().await = Some(Local::now());

        // 清理旧备份
        self.cleanup_old_backups().await?;

        Ok(backup_file)
    }

    async fn restore_backup(&self, backup_file: &Path) -> AppResult<()> {
        let file = std::fs::File::open(backup_file)?;
        let mut archive = zip::ZipArchive::new(file)?;

        // 创建临时目录
        let temp_dir = Path::new("temp_restore");
        if temp_dir.exists() {
            fs::remove_dir_all(temp_dir).await?;
        }
        fs::create_dir_all(temp_dir).await?;

        // 解压文件
        for i in 0..archive.len() {
            let mut file = archive.by_index(i)?;
            let outpath = temp_dir.join(file.name());

            if file.name().ends_with('/') {
                fs::create_dir_all(&outpath).await?;
            } else {
                if let Some(p) = outpath.parent() {
                    if !p.exists() {
                        fs::create_dir_all(p).await?;
                    }
                }
                let mut outfile = std::fs::File::create(&outpath)?;
                std::io::copy(&mut file, &mut outfile)?;
            }
        }

        // 恢复文件
        if Path::new("temp_restore/time_tracker.db").exists() {
            fs::copy("temp_restore/time_tracker.db", "time_tracker.db").await?;
        }
        if Path::new("temp_restore/config.json").exists() {
            fs::copy("temp_restore/config.json", "config.json").await?;
        }

        // 清理临时目录
        fs::remove_dir_all(temp_dir).await?;

        Ok(())
    }

    async fn list_backups(&self) -> AppResult<Vec<PathBuf>> {
        let config = self.config.read().await;
        let backup_dir = Path::new(&config.backup_dir);

        if !backup_dir.exists() {
            return Ok(vec![]);
        }

        let mut backups = vec![];
        let mut entries = fs::read_dir(backup_dir).await?;
        while let Some(entry) = entries.next_entry().await? {
            let path = entry.path();
            if path.extension().map_or(false, |ext| ext == "zip") {
                backups.push(path);
            }
        }

        backups.sort();
        Ok(backups)
    }

    async fn cleanup_old_backups(&self) -> AppResult<()> {
        let config = self.config.read().await;
        let backups = self.list_backups().await?;

        if backups.len() > config.max_backups {
            for backup in backups.iter().take(backups.len() - config.max_backups) {
                fs::remove_file(backup).await?;
            }
        }

        Ok(())
    }

    async fn check_auto_backup(&self) -> AppResult<()> {
        let config = self.config.read().await;
        if !config.auto_backup {
            return Ok(());
        }

        if let Some(last_backup) = *self.last_backup.read().await {
            let elapsed = Local::now().signed_duration_since(last_backup);
            if elapsed.to_std()? >= config.backup_interval {
                self.create_backup().await?;
            }
        } else {
            self.create_backup().await?;
        }

        Ok(())
    }
}

#[async_trait]
impl Plugin for BackupPlugin {
    fn metadata(&self) -> &PluginMetadata {
        &self.metadata
    }

    async fn initialize(&self, config: Option<serde_json::Value>) -> AppResult<()> {
        if let Some(config) = config {
            let backup_config: BackupConfig = serde_json::from_value(config)?;
            *self.config.write().await = backup_config;
        }
        Ok(())
    }

    async fn start(&self) -> AppResult<()> {
        self.check_auto_backup().await
    }

    async fn stop(&self) -> AppResult<()> {
        Ok(())
    }

    async fn on_activity_change(&self, _activity: &Activity) -> AppResult<()> {
        self.check_auto_backup().await
    }

    async fn on_pomodoro_start(&self, _session: &PomodoroSession) -> AppResult<()> {
        Ok(())
    }

    async fn on_pomodoro_end(&self, _session: &PomodoroSession) -> AppResult<()> {
        self.check_auto_backup().await
    }

    async fn on_break_start(&self, _duration: Duration) -> AppResult<()> {
        Ok(())
    }

    async fn on_break_end(&self) -> AppResult<()> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_backup_lifecycle() -> AppResult<()> {
        // 创建临时目录
        let temp_dir = tempdir()?;
        let backup_dir = temp_dir.path().join("backups");

        // 创建测试文件
        let db_content = b"test database";
        let config_content = b"test config";
        fs::write("time_tracker.db", db_content).await?;
        fs::write("config.json", config_content).await?;

        // 创建插件实例
        let plugin = BackupPlugin::new();
        
        // 配置插件
        let config = BackupConfig {
            backup_dir: backup_dir.to_string_lossy().into(),
            auto_backup: true,
            backup_interval: Duration::from_secs(1),
            max_backups: 2,
        };
        plugin.initialize(Some(serde_json::to_value(config)?)).await?;

        // 创建备份
        let backup_file = plugin.create_backup().await?;
        assert!(backup_file.exists());

        // 删除原始文件
        fs::remove_file("time_tracker.db").await?;
        fs::remove_file("config.json").await?;

        // 恢复备份
        plugin.restore_backup(&backup_file).await?;

        // 验证恢复的文件
        let restored_db = fs::read("time_tracker.db").await?;
        let restored_config = fs::read("config.json").await?;
        assert_eq!(restored_db, db_content);
        assert_eq!(restored_config, config_content);

        // 清理
        fs::remove_file("time_tracker.db").await?;
        fs::remove_file("config.json").await?;
        temp_dir.close()?;

        Ok(())
    }
} 