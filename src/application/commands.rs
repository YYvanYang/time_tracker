use crate::application::events::{AppEvent, EventBus};
use crate::application::services::ServiceContainer;
use crate::core::{AppError, AppResult};
use crate::infrastructure::config::Config;
use crate::plugins::PluginRegistry;
use std::sync::Arc;

pub struct CommandHandler {
    services: Arc<ServiceContainer>,
    event_bus: EventBus,
    plugin_registry: Arc<PluginRegistry>,
}

impl CommandHandler {
    pub fn new(
        services: Arc<ServiceContainer>,
        event_bus: EventBus,
        plugin_registry: Arc<PluginRegistry>,
    ) -> Self {
        Self {
            services,
            event_bus,
            plugin_registry,
        }
    }

    pub fn get_config(&self) -> &Config {
        &self.services.config
    }

    pub async fn update_config(&self, config: Config) -> AppResult<()> {
        // 验证配置
        if let Err(e) = config.validate() {
            return Err(AppError::Validation(e.to_string()));
        }

        // 保存配置
        self.services.config_manager.save_config(&config).await?;

        // 更新服务配置
        self.services.update_config(config.clone()).await?;

        // 通知配置更新
        self.event_bus.publish(AppEvent::ConfigUpdated);

        Ok(())
    }

    pub async fn quit(&self) -> AppResult<()> {
        // 发布应用停止事件
        self.event_bus.publish(AppEvent::ApplicationStopping);

        // 停止所有插件
        self.plugin_registry.stop_all().await?;

        // 保存当前状态
        self.services.save_state().await?;

        // 清理资源
        self.services.cleanup().await?;

        Ok(())
    }

    pub async fn show_window(&self) -> AppResult<()> {
        // 检查窗口状态
        if !self.services.window_manager.is_visible().await? {
            // 显示窗口
            self.services.window_manager.show().await?;

            // 将窗口带到前台
            self.services.platform.bring_to_front().await?;

            // 更新UI状态
            self.event_bus.publish(AppEvent::WindowShown);
        }

        Ok(())
    }

    pub async fn hide_window(&self) -> AppResult<()> {
        // 检查窗口状态
        if self.services.window_manager.is_visible().await? {
            // 隐藏窗口
            self.services.window_manager.hide().await?;

            // 更新UI状态
            self.event_bus.publish(AppEvent::WindowHidden);
        }

        Ok(())
    }

    pub async fn toggle_window(&self) -> AppResult<()> {
        if self.services.window_manager.is_visible().await? {
            self.hide_window().await
        } else {
            self.show_window().await
        }
    }
} 