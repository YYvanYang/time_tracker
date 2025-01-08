use std::sync::Arc;
use tokio::sync::broadcast;
use tokio::task::JoinHandle;

use crate::core::AppResult;
use crate::application::services::ServiceContainer;
use crate::application::events::{AppEvent, EventBus};
use crate::infrastructure::storage::SqliteStorage;
use crate::infrastructure::config::FileConfigManager;
use crate::plugins::registry::PluginRegistry;

pub struct App {
    services: Arc<ServiceContainer>,
    event_bus: EventBus,
    plugin_registry: Arc<PluginRegistry>,
    background_tasks: Vec<JoinHandle<()>>,
}

impl App {
    pub async fn new() -> AppResult<Self> {
        let storage = Arc::new(SqliteStorage::new().await?);
        let config_manager = Arc::new(FileConfigManager::new());
        let config = config_manager.load_config().await?;

        let (event_sender, _) = broadcast::channel(100);
        let event_bus = EventBus::new(event_sender.clone());
        let plugin_registry = Arc::new(PluginRegistry::new(event_sender));

        let services = Arc::new(ServiceContainer::new(
            storage,
            config,
            config_manager,
        ));

        Ok(Self {
            services,
            event_bus,
            plugin_registry,
            background_tasks: Vec::new(),
        })
    }

    pub fn get_services(&self) -> Arc<ServiceContainer> {
        self.services.clone()
    }

    pub fn get_event_bus(&self) -> EventBus {
        self.event_bus.clone()
    }

    pub fn get_plugin_registry(&self) -> Arc<PluginRegistry> {
        self.plugin_registry.clone()
    }

    pub async fn start(&mut self) -> AppResult<()> {
        self.plugin_registry.load_plugins().await?;
        Ok(())
    }

    pub async fn stop(&mut self) -> AppResult<()> {
        for task in self.background_tasks.drain(..) {
            task.abort();
        }
        self.plugin_registry.unload_plugins().await?;
        Ok(())
    }
} 