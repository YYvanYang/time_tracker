use crate::error::Result;
use crate::platform::{PlatformOperations, WindowInfo};
use std::ffi::OsString;
use std::os::windows::ffi::OsStringExt;
use std::ptr;
use winapi::um::winuser;
use winapi::shared::windef;
use winreg::enums::*;
use winreg::RegKey;
use std::path::PathBuf;
use std::env;

pub struct WindowsPlatform {
    // Windows 平台特定字段
}

impl WindowsPlatform {
    pub fn new() -> Result<Self> {
        Ok(Self {})
    }

    fn get_window_text(hwnd: windef::HWND) -> Option<String> {
        unsafe {
            // 获取窗口标题所需的长度
            let len = winuser::GetWindowTextLengthW(hwnd) + 1;
            if len <= 1 {
                return None;
            }
            
            // 创建缓冲区
            let mut buf = vec![0u16; len as usize];
            let len = winuser::GetWindowTextW(hwnd, buf.as_mut_ptr(), len);
            if len <= 0 {
                return None;
            }
            
            buf.truncate(len as usize);
            String::from_utf16_lossy(&buf).into()
        }
    }
}

impl PlatformOperations for WindowsPlatform {
    fn get_active_window(&self) -> Result<WindowInfo> {
        unsafe {
            let hwnd = winuser::GetForegroundWindow();
            if hwnd.is_null() {
                return Ok(WindowInfo::default());
            }

            let title = Self::get_window_text(hwnd).unwrap_or_default();
            
            Ok(WindowInfo {
                title,
                ..Default::default()
            })
        }
    }

    fn set_autostart(&self, enabled: bool) -> Result<()> {
        let hkcu = RegKey::predef(HKEY_CURRENT_USER);
        let path = r"Software\Microsoft\Windows\CurrentVersion\Run";
        let (key, _) = hkcu.create_subkey(path)?;
        
        let exe_path = env::current_exe()?;
        let app_name = exe_path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("MyApp");

        if enabled {
            key.set_value(app_name, &exe_path.to_str().unwrap_or_default())?;
        } else {
            key.delete_value(app_name)?;
        }
        
        Ok(())
    }

    fn is_autostart_enabled(&self) -> Result<bool> {
        let hkcu = RegKey::predef(HKEY_CURRENT_USER);
        let path = r"Software\Microsoft\Windows\CurrentVersion\Run";
        let key = hkcu.open_subkey(path)?;
        
        let exe_path = env::current_exe()?;
        let app_name = exe_path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("MyApp");

        match key.get_value::<String, _>(app_name) {
            Ok(_) => Ok(true),
            Err(_) => Ok(false),
        }
    }
} 