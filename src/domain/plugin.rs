use crate::core::{AppResult, models::*};
use crate::core::traits::Storage;
use std::sync::Arc;
use tokio::sync::RwLock;
use async_trait::async_trait;
use serde::{Serialize, Deserialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginMetadata {
    pub id: String,
    pub name: String,
    pub version: String,
    pub author: String,
    pub description: String,
    pub dependencies: Vec<String>,
    pub config_schema: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginConfig {
    pub plugin_id: String,
    pub enabled: bool,
    pub config: Option<serde_json::Value>,
}

#[async_trait]
pub trait Plugin: Send + Sync {
    fn metadata(&self) -> &PluginMetadata;
    
    async fn initialize(&self, config: Option<serde_json::Value>) -> AppResult<()>;
    async fn start(&self) -> AppResult<()>;
    async fn stop(&self) -> AppResult<()>;
    
    async fn on_activity_change(&self, activity: &Activity) -> AppResult<()>;
    async fn on_pomodoro_start(&self, session: &PomodoroSession) -> AppResult<()>;
    async fn on_pomodoro_end(&self, session: &PomodoroSession) -> AppResult<()>;
    async fn on_break_start(&self, duration: std::time::Duration) -> AppResult<()>;
    async fn on_break_end(&self) -> AppResult<()>;
}

pub struct PluginManager {
    storage: Arc<dyn Storage>,
    plugins: RwLock<HashMap<String, Arc<dyn Plugin>>>,
    configs: RwLock<HashMap<String, PluginConfig>>,
}

impl PluginManager {
    pub fn new(storage: Arc<dyn Storage>) -> Self {
        Self {
            storage,
            plugins: RwLock::new(HashMap::new()),
            configs: RwLock::new(HashMap::new()),
        }
    }

    pub async fn register_plugin(&self, plugin: Arc<dyn Plugin>) -> AppResult<()> {
        let metadata = plugin.metadata();
        let plugin_id = metadata.id.clone();

        // 检查依赖
        for dep_id in &metadata.dependencies {
            if !self.plugins.read().await.contains_key(dep_id) {
                return Err(format!("Missing dependency: {}", dep_id).into());
            }
        }

        // 获取或创建插件配置
        let config = if let Some(config) = self.storage.get_plugin_config(&plugin_id).await? {
            config
        } else {
            PluginConfig {
                plugin_id: plugin_id.clone(),
                enabled: true,
                config: None,
            }
        };

        // 初始化插件
        plugin.initialize(config.config.clone()).await?;

        // 如果插件已启用,则启动它
        if config.enabled {
            plugin.start().await?;
        }

        // 保存插件和配置
        self.plugins.write().await.insert(plugin_id.clone(), plugin);
        self.configs.write().await.insert(plugin_id, config);

        Ok(())
    }

    pub async fn unregister_plugin(&self, plugin_id: &str) -> AppResult<()> {
        if let Some(plugin) = self.plugins.write().await.remove(plugin_id) {
            plugin.stop().await?;
        }
        self.configs.write().await.remove(plugin_id);
        Ok(())
    }

    pub async fn enable_plugin(&self, plugin_id: &str) -> AppResult<()> {
        if let Some(plugin) = self.plugins.read().await.get(plugin_id) {
            if let Some(mut config) = self.configs.write().await.get_mut(plugin_id) {
                config.enabled = true;
                self.storage.save_plugin_config(&config).await?;
                plugin.start().await?;
            }
        }
        Ok(())
    }

    pub async fn disable_plugin(&self, plugin_id: &str) -> AppResult<()> {
        if let Some(plugin) = self.plugins.read().await.get(plugin_id) {
            if let Some(mut config) = self.configs.write().await.get_mut(plugin_id) {
                config.enabled = false;
                self.storage.save_plugin_config(&config).await?;
                plugin.stop().await?;
            }
        }
        Ok(())
    }

    pub async fn configure_plugin(
        &self,
        plugin_id: &str,
        config: serde_json::Value,
    ) -> AppResult<()> {
        if let Some(plugin) = self.plugins.read().await.get(plugin_id) {
            if let Some(mut plugin_config) = self.configs.write().await.get_mut(plugin_id) {
                plugin_config.config = Some(config.clone());
                self.storage.save_plugin_config(&plugin_config).await?;
                plugin.initialize(Some(config)).await?;
                if plugin_config.enabled {
                    plugin.start().await?;
                }
            }
        }
        Ok(())
    }

    pub async fn get_plugin_metadata(&self, plugin_id: &str) -> Option<PluginMetadata> {
        self.plugins.read().await
            .get(plugin_id)
            .map(|p| p.metadata().clone())
    }

    pub async fn get_plugin_config(&self, plugin_id: &str) -> Option<PluginConfig> {
        self.configs.read().await
            .get(plugin_id)
            .cloned()
    }

    pub async fn get_all_plugins(&self) -> Vec<PluginMetadata> {
        self.plugins.read().await
            .values()
            .map(|p| p.metadata().clone())
            .collect()
    }

    pub async fn notify_activity_change(&self, activity: &Activity) -> AppResult<()> {
        for plugin in self.plugins.read().await.values() {
            if let Some(config) = self.configs.read().await.get(&plugin.metadata().id) {
                if config.enabled {
                    plugin.on_activity_change(activity).await?;
                }
            }
        }
        Ok(())
    }

    pub async fn notify_pomodoro_start(&self, session: &PomodoroSession) -> AppResult<()> {
        for plugin in self.plugins.read().await.values() {
            if let Some(config) = self.configs.read().await.get(&plugin.metadata().id) {
                if config.enabled {
                    plugin.on_pomodoro_start(session).await?;
                }
            }
        }
        Ok(())
    }

    pub async fn notify_pomodoro_end(&self, session: &PomodoroSession) -> AppResult<()> {
        for plugin in self.plugins.read().await.values() {
            if let Some(config) = self.configs.read().await.get(&plugin.metadata().id) {
                if config.enabled {
                    plugin.on_pomodoro_end(session).await?;
                }
            }
        }
        Ok(())
    }

    pub async fn notify_break_start(&self, duration: std::time::Duration) -> AppResult<()> {
        for plugin in self.plugins.read().await.values() {
            if let Some(config) = self.configs.read().await.get(&plugin.metadata().id) {
                if config.enabled {
                    plugin.on_break_start(duration).await?;
                }
            }
        }
        Ok(())
    }

    pub async fn notify_break_end(&self) -> AppResult<()> {
        for plugin in self.plugins.read().await.values() {
            if let Some(config) = self.configs.read().await.get(&plugin.metadata().id) {
                if config.enabled {
                    plugin.on_break_end().await?;
                }
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use mockall::mock;
    use mockall::predicate::*;
    use std::time::Duration;

    struct TestPlugin {
        metadata: PluginMetadata,
        initialized: RwLock<bool>,
        started: RwLock<bool>,
    }

    #[async_trait]
    impl Plugin for TestPlugin {
        fn metadata(&self) -> &PluginMetadata {
            &self.metadata
        }

        async fn initialize(&self, _config: Option<serde_json::Value>) -> AppResult<()> {
            *self.initialized.write().await = true;
            Ok(())
        }

        async fn start(&self) -> AppResult<()> {
            *self.started.write().await = true;
            Ok(())
        }

        async fn stop(&self) -> AppResult<()> {
            *self.started.write().await = false;
            Ok(())
        }

        async fn on_activity_change(&self, _activity: &Activity) -> AppResult<()> {
            Ok(())
        }

        async fn on_pomodoro_start(&self, _session: &PomodoroSession) -> AppResult<()> {
            Ok(())
        }

        async fn on_pomodoro_end(&self, _session: &PomodoroSession) -> AppResult<()> {
            Ok(())
        }

        async fn on_break_start(&self, _duration: Duration) -> AppResult<()> {
            Ok(())
        }

        async fn on_break_end(&self) -> AppResult<()> {
            Ok(())
        }
    }

    mock! {
        Storage {}
        #[async_trait::async_trait]
        impl Storage for Storage {
            async fn get_plugin_config(&self, plugin_id: &str) -> AppResult<Option<PluginConfig>>;
            async fn save_plugin_config(&self, config: &PluginConfig) -> AppResult<()>;
        }
    }

    #[tokio::test]
    async fn test_plugin_lifecycle() -> AppResult<()> {
        let mut mock_storage = MockStorage::new();
        
        // 设置模拟数据
        mock_storage
            .expect_get_plugin_config()
            .returning(|_| Ok(None));
        
        mock_storage
            .expect_save_plugin_config()
            .returning(|_| Ok(()));

        let manager = PluginManager::new(Arc::new(mock_storage));

        // 创建测试插件
        let test_plugin = Arc::new(TestPlugin {
            metadata: PluginMetadata {
                id: "test_plugin".into(),
                name: "Test Plugin".into(),
                version: "1.0.0".into(),
                author: "Test Author".into(),
                description: "Test Description".into(),
                dependencies: vec![],
                config_schema: None,
            },
            initialized: RwLock::new(false),
            started: RwLock::new(false),
        });

        // 注册插件
        manager.register_plugin(test_plugin.clone()).await?;

        // 验证插件状态
        assert!(*test_plugin.initialized.read().await);
        assert!(*test_plugin.started.read().await);

        // 禁用插件
        manager.disable_plugin("test_plugin").await?;
        assert!(!*test_plugin.started.read().await);

        // 启用插件
        manager.enable_plugin("test_plugin").await?;
        assert!(*test_plugin.started.read().await);

        Ok(())
    }
} 