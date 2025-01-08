use std::path::PathBuf;
use async_trait::async_trait;
use tokio::fs;
use crate::core::AppResult;
use crate::domain::config::{AppConfig, ConfigManager};

pub struct FileConfigManager {
    config_path: PathBuf,
}

impl FileConfigManager {
    pub fn new() -> Self {
        let config_dir = dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("./config"))
            .join("time_tracker");
        
        fs::create_dir_all(&config_dir).unwrap_or_default();
        
        Self {
            config_path: config_dir.join("config.json"),
        }
    }
}

#[async_trait]
impl ConfigManager for FileConfigManager {
    async fn save_config(&self, config: &AppConfig) -> AppResult<()> {
        let json = serde_json::to_string_pretty(config)?;
        fs::write(&self.config_path, json).await?;
        Ok(())
    }

    async fn load_config(&self) -> AppResult<AppConfig> {
        match fs::read_to_string(&self.config_path).await {
            Ok(content) => {
                let config = serde_json::from_str(&content)?;
                Ok(config)
            }
            Err(_) => Ok(AppConfig::default())
        }
    }

    async fn get_config(&self) -> AppResult<AppConfig> {
        self.load_config().await
    }

    async fn update_config(&self, config: AppConfig) -> AppResult<()> {
        self.save_config(&config).await
    }
} 