use crate::core::AppResult;
use crate::plugins::traits::Plugin;
use async_trait::async_trait;
use serde_json::Value;

pub struct StatsPlugin;

impl StatsPlugin {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl Plugin for StatsPlugin {
    fn name(&self) -> &str {
        "统计插件"
    }

    fn version(&self) -> &str {
        "1.0.0"
    }

    fn description(&self) -> &str {
        "提供基本的统计功能"
    }

    async fn initialize(&self) -> AppResult<()> {
        Ok(())
    }

    async fn start(&self) -> AppResult<()> {
        Ok(())
    }

    async fn stop(&self) -> AppResult<()> {
        Ok(())
    }

    async fn uninstall(&self) -> AppResult<()> {
        Ok(())
    }

    fn get_settings_ui(&self) -> Option<Box<dyn std::any::Any>> {
        None
    }
} 