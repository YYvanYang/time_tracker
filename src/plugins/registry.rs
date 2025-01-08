use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use tokio::sync::broadcast;
use crate::core::AppResult;
use crate::plugins::loader::PluginLoader;
use crate::plugins::traits::Plugin;

pub enum PluginEvent {
    Loaded(String),
    Unloaded(String),
    Error(String),
}

pub struct PluginRegistry {
    plugins: RwLock<HashMap<String, Arc<dyn Plugin>>>,
    loader: RwLock<PluginLoader>,
    event_sender: broadcast::Sender<PluginEvent>,
}

impl PluginRegistry {
    pub fn new(event_sender: broadcast::Sender<PluginEvent>) -> Self {
        Self {
            plugins: RwLock::new(HashMap::new()),
            loader: RwLock::new(PluginLoader::new()),
            event_sender,
        }
    }

    pub async fn load_plugin(&self, plugin_name: &str) -> AppResult<()> {
        let plugin = {
            let mut loader = self.loader.write().unwrap();
            loader.load_plugin(plugin_name)?
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
            loader.unload_plugin(plugin_name)?;
        }

        self.event_sender
            .send(PluginEvent::Unloaded(plugin_name.to_string()))
            .unwrap_or_default();

        Ok(())
    }

    pub async fn reload_plugin(&self, plugin_name: &str) -> AppResult<()> {
        self.unload_plugin(plugin_name).await?;
        self.load_plugin(plugin_name).await?;
        Ok(())
    }

    pub async fn get_plugin(&self, plugin_name: &str) -> Option<Arc<dyn Plugin>> {
        let plugins = self.plugins.read().unwrap();
        plugins.get(plugin_name).cloned()
    }

    pub async fn get_plugins(&self) -> Vec<(String, Arc<dyn Plugin>)> {
        let plugins = self.plugins.read().unwrap();
        plugins
            .iter()
            .map(|(name, plugin)| (name.clone(), plugin.clone()))
            .collect()
    }

    pub async fn load_plugins(&self) -> AppResult<()> {
        let plugin_names = {
            let loader = self.loader.read().unwrap();
            loader.list_plugins()?
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

    pub async fn unload_plugins(&self) -> AppResult<()> {
        let plugin_names: Vec<String> = {
            let plugins = self.plugins.read().unwrap();
            plugins.keys().cloned().collect()
        };

        for name in plugin_names {
            if let Err(e) = self.unload_plugin(&name).await {
                self.event_sender
                    .send(PluginEvent::Error(format!("卸载插件 {} 失败: {}", name, e)))
                    .unwrap_or_default();
            }
        }

        Ok(())
    }
} 