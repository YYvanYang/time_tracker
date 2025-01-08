use std::sync::Arc;
use crate::core::{AppResult, error::AppError, traits::Storage};
use crate::infrastructure::config::Config;
use crate::domain::config::ConfigManager;

pub struct ServiceContainer {
    pub storage: Arc<dyn Storage + Send + Sync>,
    pub config: Config,
    pub config_manager: Arc<dyn ConfigManager + Send + Sync>,
}

impl ServiceContainer {
    pub fn new(
        storage: Arc<dyn Storage + Send + Sync>,
        config: Config,
        config_manager: Arc<dyn ConfigManager + Send + Sync>,
    ) -> Self {
        Self {
            storage,
            config,
            config_manager,
        }
    }

    pub async fn update_config(&mut self, config: Config) -> AppResult<()> {
        self.config = config;
        Ok(())
    }
}

#[async_trait::async_trait]
pub trait WindowManager: Send + Sync {
    async fn show(&self) -> AppResult<()>;
    async fn hide(&self) -> AppResult<()>;
    async fn is_visible(&self) -> AppResult<bool>;
} 