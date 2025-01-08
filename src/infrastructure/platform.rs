use async_trait::async_trait;
use crate::core::AppResult;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DockIconVisibility {
    Show,
    Hide,
}

#[derive(Debug, Clone)]
pub struct NotificationOptions {
    pub title: String,
    pub body: String,
    pub sound: bool,
    pub action_url: Option<String>,
}

#[async_trait]
pub trait PlatformOperations: Send + Sync {
    // 窗口操作
    async fn bring_to_front(&self) -> AppResult<()>;
    async fn set_dock_icon_visibility(&self, visibility: DockIconVisibility) -> AppResult<()>;
    
    // 系统通知
    async fn show_notification(&self, options: NotificationOptions) -> AppResult<()>;
    
    // 快捷键
    async fn register_global_shortcut(&self, shortcut: &str, id: &str) -> AppResult<()>;
    async fn unregister_global_shortcut(&self, id: &str) -> AppResult<()>;
    
    // 自启动
    async fn set_auto_start(&self, enabled: bool) -> AppResult<()>;
    async fn is_auto_start_enabled(&self) -> AppResult<bool>;
    
    // 系统状态
    async fn is_screen_locked(&self) -> AppResult<bool>;
    async fn prevent_system_sleep(&self, prevent: bool) -> AppResult<()>;
    
    // 应用程序
    async fn get_active_window_info(&self) -> AppResult<(String, String)>;
    async fn get_idle_duration(&self) -> AppResult<std::time::Duration>;
} 