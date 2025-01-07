use crate::core::{AppError, AppResult};

#[derive(Debug, Clone)]
pub struct WindowInfo {
    pub title: String,
    pub process_name: String,
    pub process_id: u32,
    pub app_name: String,
    pub window_title: String,
}

#[derive(Debug, Clone)]
pub enum DockIconVisibility {
    Visible,
    Hidden,
}

#[derive(Debug, Clone)]
pub struct NotificationOptions {
    pub title: String,
    pub message: String,
    pub sound: bool,
    pub action_button: Option<String>,
    pub cancel_button: Option<String>,
}

pub trait PlatformOperations: Send + Sync {
    // 基本窗口操作
    fn get_active_window(&self) -> AppResult<WindowInfo>;
    fn set_autostart(&self, enabled: bool) -> AppResult<()>;
    fn is_autostart_enabled(&self) -> AppResult<bool>;

    // Dock 图标管理
    fn set_dock_icon_visibility(&self, visibility: DockIconVisibility) -> AppResult<()> {
        Err(AppError::Platform("Operation not supported on this platform".into()))
    }

    fn get_dock_icon_visibility(&self) -> AppResult<DockIconVisibility> {
        Err(AppError::Platform("Operation not supported on this platform".into()))
    }

    // 窗口管理
    fn bring_to_front(&self) -> AppResult<()> {
        Err(AppError::Platform("Operation not supported on this platform".into()))
    }

    fn hide_window(&self) -> AppResult<()> {
        Err(AppError::Platform("Operation not supported on this platform".into()))
    }

    fn show_window(&self) -> AppResult<()> {
        Err(AppError::Platform("Operation not supported on this platform".into()))
    }

    // 系统通知
    fn show_notification(&self, options: NotificationOptions) -> AppResult<()> {
        Err(AppError::Platform("Operation not supported on this platform".into()))
    }

    fn request_notification_permissions(&self) -> AppResult<()> {
        Err(AppError::Platform("Operation not supported on this platform".into()))
    }

    // 全局快捷键
    fn register_global_shortcut(&self, shortcut: &str, id: &str) -> AppResult<()> {
        Err(AppError::Platform("Operation not supported on this platform".into()))
    }

    fn unregister_global_shortcut(&self, id: &str) -> AppResult<()> {
        Err(AppError::Platform("Operation not supported on this platform".into()))
    }

    // 系统空闲时间
    fn get_system_idle_time(&self) -> AppResult<std::time::Duration> {
        Err(AppError::Platform("Operation not supported on this platform".into()))
    }

    // 系统主题
    fn get_system_theme(&self) -> AppResult<String> {
        Err(AppError::Platform("Operation not supported on this platform".into()))
    }

    // 系统电源状态
    fn is_on_battery(&self) -> AppResult<bool> {
        Err(AppError::Platform("Operation not supported on this platform".into()))
    }

    // 系统休眠预防
    fn prevent_system_sleep(&self, prevent: bool) -> AppResult<()> {
        Err(AppError::Platform("Operation not supported on this platform".into()))
    }
}

#[cfg(target_os = "windows")]
mod windows;
#[cfg(target_os = "windows")]
pub use self::windows::WindowsPlatform;

#[cfg(target_os = "macos")]
mod macos;
#[cfg(target_os = "macos")]
pub use self::macos::MacOSPlatform;

pub fn init() -> AppResult<Box<dyn PlatformOperations + Send + Sync>> {
    #[cfg(target_os = "windows")]
    {
        Ok(Box::new(WindowsPlatform::new()?))
    }
    #[cfg(target_os = "macos")]
    {
        Ok(Box::new(MacOSPlatform::new()?))
    }
    #[cfg(not(any(target_os = "windows", target_os = "macos")))]
    {
        Err(AppError::Platform("Platform not supported".into()))
    }
} 