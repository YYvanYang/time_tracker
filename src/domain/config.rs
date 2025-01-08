use crate::core::{AppResult, models::*};
use crate::core::traits::Storage;
use std::sync::Arc;
use tokio::sync::RwLock;
use serde::{Serialize, Deserialize};
use std::time::Duration;
use async_trait::async_trait;

#[async_trait]
pub trait ConfigManager: Send + Sync {
    async fn save_config(&self, config: &AppConfig) -> AppResult<()>;
    async fn load_config(&self) -> AppResult<AppConfig>;
    async fn get_config(&self) -> AppResult<AppConfig>;
    async fn update_config(&self, config: AppConfig) -> AppResult<()>;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub pomodoro: PomodoroSettings,
    pub notification: NotificationSettings,
    pub ui: UISettings,
    pub storage: StorageSettings,
    pub rules: RuleSettings,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PomodoroSettings {
    pub work_duration: Duration,
    pub short_break_duration: Duration,
    pub long_break_duration: Duration,
    pub long_break_interval: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationSettings {
    pub enable_system_notifications: bool,
    pub enable_sound: bool,
    pub sound_volume: f32,
    pub notification_retention_days: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UISettings {
    pub theme: String,
    pub language: String,
    pub show_system_tray: bool,
    pub minimize_to_tray: bool,
    pub start_minimized: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageSettings {
    pub database_path: String,
    pub backup_path: String,
    pub auto_backup: bool,
    pub backup_interval_days: u32,
    pub backup_retention_days: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuleSettings {
    pub auto_categorize: bool,
    pub productivity_threshold: f64,
    pub min_activity_duration: Duration,
    pub suggestion_threshold: u32,
}

pub struct ConfigManagerImpl {
    storage: Arc<dyn Storage>,
    config: RwLock<AppConfig>,
}

impl ConfigManagerImpl {
    pub fn new(storage: Arc<dyn Storage>) -> Self {
        Self {
            storage,
            config: RwLock::new(AppConfig::default()),
        }
    }
}

#[async_trait]
impl ConfigManager for ConfigManagerImpl {
    async fn save_config(&self, config: &AppConfig) -> AppResult<()> {
        *self.config.write().await = config.clone();
        Ok(())
    }

    async fn load_config(&self) -> AppResult<AppConfig> {
        Ok(self.config.read().await.clone())
    }

    async fn get_config(&self) -> AppResult<AppConfig> {
        Ok(self.config.read().await.clone())
    }

    async fn update_config(&self, config: AppConfig) -> AppResult<()> {
        *self.config.write().await = config;
        Ok(())
    }
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            pomodoro: PomodoroSettings {
                work_duration: Duration::from_secs(25 * 60),
                short_break_duration: Duration::from_secs(5 * 60),
                long_break_duration: Duration::from_secs(15 * 60),
                long_break_interval: 4,
            },
            notification: NotificationSettings {
                enable_system_notifications: true,
                enable_sound: true,
                sound_volume: 0.7,
                notification_retention_days: 30,
            },
            ui: UISettings {
                theme: "system".into(),
                language: "zh-CN".into(),
                show_system_tray: true,
                minimize_to_tray: true,
                start_minimized: false,
            },
            storage: StorageSettings {
                database_path: "time_tracker.db".into(),
                backup_path: "backups".into(),
                auto_backup: true,
                backup_interval_days: 7,
                backup_retention_days: 30,
            },
            rules: RuleSettings {
                auto_categorize: true,
                productivity_threshold: 0.7,
                min_activity_duration: Duration::from_secs(60),
                suggestion_threshold: 10,
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use mockall::predicate::*;
    use mockall::mock;

    mock! {
        Storage {}
        #[async_trait]
        impl Storage for Storage {
            async fn get_config(&self) -> AppResult<Option<AppConfig>>;
            async fn save_config(&self, config: &AppConfig) -> AppResult<()>;
        }
    }

    #[tokio::test]
    async fn test_config_lifecycle() -> AppResult<()> {
        let mut mock_storage = MockStorage::new();
        
        mock_storage
            .expect_get_config()
            .returning(|| Ok(Some(AppConfig::default())));
        
        mock_storage
            .expect_save_config()
            .returning(|_| Ok(()));

        let manager = ConfigManagerImpl::new(Arc::new(mock_storage));

        // 测试加载配置
        let config = manager.get_config().await?;
        assert_eq!(config.ui.language, "zh-CN");

        // 测试更新配置
        let mut new_config = config.clone();
        new_config.ui.language = "en-US".into();
        manager.update_config(new_config.clone()).await?;

        let updated_config = manager.get_config().await?;
        assert_eq!(updated_config.ui.language, "en-US");

        Ok(())
    }
} 