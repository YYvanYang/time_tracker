use crate::core::AppResult;
use crate::plugins::traits::{Plugin, NotificationPlugin};
use async_trait::async_trait;
use notify_rust::Notification;
use std::any::Any;

pub struct SystemNotificationPlugin;

impl SystemNotificationPlugin {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl Plugin for SystemNotificationPlugin {
    fn name(&self) -> &str {
        "system_notification"
    }

    fn version(&self) -> &str {
        "1.0.0"
    }

    fn description(&self) -> &str {
        "系统通知插件"
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

    fn get_settings_ui(&self) -> Option<Box<dyn Any>> {
        None
    }
}

#[async_trait]
impl NotificationPlugin for SystemNotificationPlugin {
    async fn send_notification(&self, title: &str, message: &str) -> AppResult<()> {
        Notification::new()
            .summary(title)
            .body(message)
            .icon("time-tracker")
            .timeout(5000) // 5 seconds
            .show()?;
            
        Ok(())
    }
} 