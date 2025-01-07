use crate::core::AppResult;
use crate::plugins::loader::PluginLoader;
use crate::plugins::traits::Plugin;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, RwLock};
use tokio::sync::broadcast;

pub struct PluginRegistry {
    plugins: RwLock<HashMap<String, Arc<dyn Plugin>>>,
    loader: RwLock<PluginLoader>,
    event_sender: broadcast::Sender<PluginEvent>,
}

#[derive(Debug, Clone)]
pub enum PluginEvent {
    Loaded(String),
    Unloaded(String),
    Reloaded(String),
    Error(String),
}

impl PluginRegistry {
    pub fn new(plugin_dir: PathBuf) -> Self {
        let (tx, _) = broadcast::channel(100);
        Self {
            plugins: RwLock::new(HashMap::new()),
            loader: RwLock::new(PluginLoader::new(plugin_dir)),
            event_sender: tx,
        }
    }

    pub fn subscribe(&self) -> broadcast::Receiver<PluginEvent> {
        self.event_sender.subscribe()
    }

    pub async fn load_plugin(&self, plugin_name: &str) -> AppResult<()> {
        let plugin = {
            let mut loader = self.loader.write().unwrap();
            loader.load_plugin(plugin_name).await?
        };

        {
            let mut plugins = self.plugins.write().unwrap();
            plugins.insert(plugin_name.to_string(), plugin);
        }

        self.event_sender
            .send(PluginEvent::Loaded(plugin_name.to_string()))
            .unwrap_or_default();

        Ok(())
    }

    pub async fn unload_plugin(&self, plugin_name: &str) -> AppResult<()> {
        {
            let mut plugins = self.plugins.write().unwrap();
            plugins.remove(plugin_name);
        }

        {
            let mut loader = self.loader.write().unwrap();
            loader.unload_plugin(plugin_name).await?;
        }

        self.event_sender
            .send(PluginEvent::Unloaded(plugin_name.to_string()))
            .unwrap_or_default();

        Ok(())
    }

    pub async fn reload_plugin(&self, plugin_name: &str) -> AppResult<()> {
        let plugin = {
            let mut loader = self.loader.write().unwrap();
            loader.reload_plugin(plugin_name).await?
        };

        {
            let mut plugins = self.plugins.write().unwrap();
            plugins.insert(plugin_name.to_string(), plugin);
        }

        self.event_sender
            .send(PluginEvent::Reloaded(plugin_name.to_string()))
            .unwrap_or_default();

        Ok(())
    }

    pub async fn scan_and_load_plugins(&self) -> AppResult<()> {
        let plugin_names = {
            let loader = self.loader.read().unwrap();
            loader.scan_plugins().await?
        };

        for name in plugin_names {
            if let Err(e) = self.load_plugin(&name).await {
                self.event_sender
                    .send(PluginEvent::Error(format!("加载插件 {} 失败: {}", name, e)))
                    .unwrap_or_default();
            }
        }

        Ok(())
    }

    pub fn list_plugins(&self) -> Vec<Arc<dyn Plugin>> {
        let plugins = self.plugins.read().unwrap();
        plugins.values().cloned().collect()
    }

    pub async fn start_all(&self) -> AppResult<()> {
        let plugins = self.plugins.read().unwrap();
        for plugin in plugins.values() {
            plugin.start().await?;
        }
        Ok(())
    }

    pub async fn stop_all(&self) -> AppResult<()> {
        let plugins = self.plugins.read().unwrap();
        for plugin in plugins.values() {
            plugin.stop().await?;
        }
        Ok(())
    }
} 