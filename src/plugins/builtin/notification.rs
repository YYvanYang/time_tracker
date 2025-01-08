use crate::core::AppResult;
use crate::plugins::traits::Plugin;
use async_trait::async_trait;
use notify_rust::Notification;

pub struct NotificationPlugin;

impl NotificationPlugin {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl Plugin for NotificationPlugin {
    fn name(&self) -> &str {
        "通知插件"
    }

    fn version(&self) -> &str {
        "1.0.0"
    }

    fn description(&self) -> &str {
        "提供系统通知功能"
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

impl NotificationPlugin {
    pub async fn send_notification(&self, title: &str, message: &str) -> AppResult<()> {
        Notification::new()
            .summary(title)
            .body(message)
            .show()?;
        Ok(())
    }
} 