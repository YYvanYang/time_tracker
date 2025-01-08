use std::sync::Arc;
use crate::core::{AppResult, error::AppError, traits::Storage, models::*};
use crate::infrastructure::config::Config;
use chrono::{DateTime, Local};

pub struct Services {
    pub storage: Arc<dyn Storage + Send + Sync>,
    pub config: Config,
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

impl Services {
    pub fn new(
        storage: Arc<dyn Storage + Send + Sync>,
        config: Config,
    ) -> Self {
        Self {
            storage,
            config,
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