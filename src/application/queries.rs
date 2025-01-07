use crate::application::services::ServiceContainer;
use crate::core::AppResult;
use crate::plugins::PluginRegistry;
use std::sync::Arc;

pub struct QueryHandler {
    services: Arc<ServiceContainer>,
    plugin_registry: Arc<PluginRegistry>,
}

impl QueryHandler {
    pub fn new(
        services: Arc<ServiceContainer>,
        plugin_registry: Arc<PluginRegistry>,
    ) -> Self {
        Self {
            services,
            plugin_registry,
        }
    }

    pub async fn get_config(&self) -> &Config {
        &self.services.config
    }

    pub async fn export_activities_csv(
        &self,
        start: chrono::DateTime<chrono::Local>,
        end: chrono::DateTime<chrono::Local>,
    ) -> AppResult<()> {
        // TODO: 实现CSV导出逻辑
        Ok(())
    }

    pub async fn export_json(
        &self,
        start: chrono::DateTime<chrono::Local>,
        end: chrono::DateTime<chrono::Local>,
    ) -> AppResult<()> {
        // TODO: 实现JSON导出逻辑
        Ok(())
    }
} 