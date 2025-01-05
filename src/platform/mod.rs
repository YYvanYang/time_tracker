use crate::error::Result;

#[derive(Debug, Clone)]
pub struct WindowInfo {
    pub title: String,
    pub process_name: String,
    pub process_id: u32,
    pub app_name: String,
    pub window_title: String,
}

pub trait PlatformOperations {
    fn get_active_window(&self) -> Result<WindowInfo>;
    fn set_autostart(&self, enabled: bool) -> Result<()>;
    fn is_autostart_enabled(&self) -> Result<bool>;
}

pub fn init() -> Result<impl PlatformOperations> {
    #[cfg(target_os = "windows")]
    {
        Ok(windows::WindowsPlatform::new()?)
    }
    #[cfg(target_os = "macos")]
    {
        Ok(macos::MacOSPlatform::new()?)
    }
    #[cfg(target_os = "linux")]
    {
        Ok(linux::LinuxPlatform::new()?)
    }
} 