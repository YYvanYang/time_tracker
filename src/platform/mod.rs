use crate::error::Result;
use crate::TimeTrackerError;

#[derive(Debug, Clone)]
pub struct WindowInfo {
    pub title: String,
    pub process_name: String,
    pub process_id: u32,
    pub app_name: String,
    pub window_title: String,
}

pub trait PlatformOperations: Send {
    fn get_active_window(&self) -> Result<WindowInfo>;
    fn set_autostart(&self, enabled: bool) -> Result<()>;
    fn is_autostart_enabled(&self) -> Result<bool>;
}

#[cfg(target_os = "windows")]
mod windows;
#[cfg(target_os = "windows")]
pub use self::windows::WindowsPlatform;

pub fn init() -> Result<Box<dyn PlatformOperations>> {
    #[cfg(target_os = "windows")]
    {
        Ok(Box::new(WindowsPlatform::new()?))
    }
    #[cfg(not(target_os = "windows"))]
    {
        Err(TimeTrackerError::Platform("Platform not supported".into()))
    }
} 