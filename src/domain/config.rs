use crate::core::{AppResult, models::*};
use crate::core::traits::Storage;
use std::sync::Arc;
use tokio::sync::RwLock;
use serde::{Serialize, Deserialize};
use std::time::Duration;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub pomodoro: PomodoroSettings,
    pub notification: NotificationSettings,
    pub ui: UISettings,
    pub storage: StorageSettings,
    pub rules: RuleSettings,
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
    pub productivity_threshold: f32,
    pub min_activity_duration: Duration,
    pub suggestion_threshold: u32,
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

pub struct ConfigManager {
    storage: Arc<dyn Storage>,
    config: RwLock<AppConfig>,
}

impl ConfigManager {
    pub fn new(storage: Arc<dyn Storage>) -> Self {
        Self {
            storage,
            config: RwLock::new(AppConfig::default()),
        }
    }

    pub async fn load_config(&self) -> AppResult<()> {
        if let Some(config) = self.storage.get_config().await? {
            *self.config.write().await = config;
        }
        Ok(())
    }

    pub async fn save_config(&self) -> AppResult<()> {
        let config = self.config.read().await;
        self.storage.save_config(&config).await
    }

    pub async fn get_config(&self) -> AppConfig {
        self.config.read().await.clone()
    }

    pub async fn update_config(&self, config: AppConfig) -> AppResult<()> {
        *self.config.write().await = config;
        self.save_config().await
    }

    pub async fn update_pomodoro_settings(&self, settings: PomodoroSettings) -> AppResult<()> {
        let mut config = self.config.write().await;
        config.pomodoro = settings;
        self.storage.save_config(&config).await
    }

    pub async fn update_notification_settings(&self, settings: NotificationSettings) -> AppResult<()> {
        let mut config = self.config.write().await;
        config.notification = settings;
        self.storage.save_config(&config).await
    }

    pub async fn update_ui_settings(&self, settings: UISettings) -> AppResult<()> {
        let mut config = self.config.write().await;
        config.ui = settings;
        self.storage.save_config(&config).await
    }

    pub async fn update_storage_settings(&self, settings: StorageSettings) -> AppResult<()> {
        let mut config = self.config.write().await;
        config.storage = settings;
        self.storage.save_config(&config).await
    }

    pub async fn update_rule_settings(&self, settings: RuleSettings) -> AppResult<()> {
        let mut config = self.config.write().await;
        config.rules = settings;
        self.storage.save_config(&config).await
    }

    pub async fn export_config(&self) -> AppResult<String> {
        let config = self.config.read().await;
        Ok(serde_json::to_string_pretty(&*config)?)
    }

    pub async fn import_config(&self, json: &str) -> AppResult<()> {
        let config: AppConfig = serde_json::from_str(json)?;
        self.update_config(config).await
    }

    pub async fn reset_to_default(&self) -> AppResult<()> {
        self.update_config(AppConfig::default()).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use mockall::mock;
    use mockall::predicate::*;

    mock! {
        Storage {}
        #[async_trait::async_trait]
        impl Storage for Storage {
            async fn get_config(&self) -> AppResult<Option<AppConfig>>;
            async fn save_config(&self, config: &AppConfig) -> AppResult<()>;
        }
    }

    #[tokio::test]
    async fn test_config_lifecycle() -> AppResult<()> {
        let mut mock_storage = MockStorage::new();
        
        // 设置模拟数据
        mock_storage
            .expect_get_config()
            .returning(|| Ok(Some(AppConfig::default())));
        
        mock_storage
            .expect_save_config()
            .returning(|_| Ok(()));

        let manager = ConfigManager::new(Arc::new(mock_storage));

        // 测试加载配置
        manager.load_config().await?;
        let config = manager.get_config().await;
        assert_eq!(config.ui.language, "zh-CN");

        // 测试更新配置
        let mut new_config = config.clone();
        new_config.ui.language = "en-US".into();
        manager.update_config(new_config).await?;

        let updated_config = manager.get_config().await;
        assert_eq!(updated_config.ui.language, "en-US");

        Ok(())
    }
} 