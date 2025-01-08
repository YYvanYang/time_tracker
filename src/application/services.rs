use crate::infrastructure::config::Config;
use crate::infrastructure::platform::PlatformOperations;
use crate::infrastructure::storage::Storage;
use crate::core::AppResult;
use std::sync::Arc;

pub struct ServiceContainer {
    pub config: Config,
    pub config_manager: Arc<dyn ConfigManager + Send + Sync>,
    pub window_manager: Arc<dyn WindowManager + Send + Sync>,
    pub platform: Arc<dyn PlatformOperations + Send + Sync>,
    pub storage: Arc<dyn Storage + Send + Sync>,
}

#[async_trait::async_trait]
pub trait ConfigManager: Send + Sync {
    async fn save_config(&self, config: &Config) -> AppResult<()>;
    async fn load_config(&self) -> AppResult<Config>;
}

#[async_trait::async_trait]
pub trait WindowManager: Send + Sync {
    async fn show(&self) -> AppResult<()>;
    async fn hide(&self) -> AppResult<()>;
    async fn is_visible(&self) -> AppResult<bool>;
}

impl ServiceContainer {
    pub fn new(
        config: Config,
        config_manager: Arc<dyn ConfigManager + Send + Sync>,
        window_manager: Arc<dyn WindowManager + Send + Sync>,
        platform: Arc<dyn PlatformOperations + Send + Sync>,
        storage: Arc<dyn Storage + Send + Sync>,
    ) -> Self {
        Self {
            config,
            config_manager,
            window_manager,
            platform,
            storage,
        }
    }

    pub async fn update_config(&self, config: Config) -> AppResult<()> {
        self.config_manager.save_config(&config).await
    }

    pub async fn save_state(&self) -> AppResult<()> {
        self.config_manager.save_config(&self.config).await
    }

    pub async fn cleanup(&self) -> AppResult<()> {
        self.save_state().await
    }
} 