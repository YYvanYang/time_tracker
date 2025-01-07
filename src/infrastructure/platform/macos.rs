use crate::core::{AppError, AppResult};
use super::{PlatformOperations, WindowInfo, DockIconVisibility, NotificationOptions};
use core_foundation::{
    base::{CFRelease, CFType, TCFType},
    array::{CFArrayGetCount, CFArrayGetValueAtIndex, CFArrayRef},
    dictionary::{CFDictionaryGetValue, CFDictionaryRef},
    number::{CFNumberGetValue, CFNumberRef},
    string::{CFStringGetCString, CFStringRef},
    url::CFURL,
};
use std::{ffi::{c_void, CStr}, ptr, path::PathBuf, time::Duration};
use objc::{
    runtime::{Object, Class, BOOL, YES, NO},
    msg_send,
};

#[link(name = "AppKit", kind = "framework")]
extern "C" {}

#[link(name = "ApplicationServices", kind = "framework")]
extern "C" {
    fn CGWindowListCopyWindowInfo(
        option: CGWindowListOption,
        relativeToWindow: CGWindowID,
    ) -> CFArrayRef;
}

#[link(name = "CoreServices", kind = "framework")]
extern "C" {
    fn CGRequestScreenCaptureAccess() -> BOOL;
    fn CGSessionCopyCurrentDictionary() -> CFDictionaryRef;
}

type CGWindowID = u32;

#[repr(u32)]
#[derive(Debug, Clone, Copy)]
enum CGWindowListOption {
    OptionOnScreenOnly = 1,
    OptionIncludingWindow = 1 << 1,
    ExcludeDesktopElements = 1 << 2,
}

const kCGNullWindowID: CGWindowID = 0;
const kCGWindowLayer: &str = "kCGWindowLayer";
const kCGWindowOwnerPID: &str = "kCGWindowOwnerPID";
const kCGWindowName: &str = "kCGWindowName";
const kCGWindowOwnerName: &str = "kCGWindowOwnerName";
const kCGWindowNumber: &str = "kCGWindowNumber";
const IDLE_TIME_THRESHOLD: Duration = Duration::from_secs(300); // 5 minutes

pub struct MacOSPlatform {
    launch_agent_path: PathBuf,
    notification_center: Option<*mut Object>,
    global_shortcuts: std::collections::HashMap<String, Box<dyn Fn() + Send + Sync>>,
}

impl MacOSPlatform {
    pub fn new() -> AppResult<Self> {
        let home_dir = dirs::home_dir()
            .ok_or_else(|| AppError::Platform("Failed to get home directory".into()))?;
        
        let launch_agent_path = home_dir
            .join("Library")
            .join("LaunchAgents")
            .join("com.timetracker.app.plist");

        Ok(Self {
            launch_agent_path,
            notification_center: None,
            global_shortcuts: std::collections::HashMap::new(),
        })
    }

    fn create_launch_agent(&self) -> AppResult<()> {
        let executable_path = std::env::current_exe()?;
        let plist_content = format!(
            r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>Label</key>
    <string>com.timetracker.app</string>
    <key>ProgramArguments</key>
    <array>
        <string>{}</string>
    </array>
    <key>RunAtLoad</key>
    <true/>
    <key>KeepAlive</key>
    <false/>
    <key>ProcessType</key>
    <string>Interactive</string>
    <key>StandardOutPath</key>
    <string>{}/Library/Logs/TimeTracker/timetracker.log</string>
    <key>StandardErrorPath</key>
    <string>{}/Library/Logs/TimeTracker/timetracker.error.log</string>
</dict>
</plist>"#,
            executable_path.to_string_lossy(),
            dirs::home_dir().unwrap().to_string_lossy(),
            dirs::home_dir().unwrap().to_string_lossy(),
        );

        // 确保目录存在
        if let Some(parent) = self.launch_agent_path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        // 创建日志目录
        let log_dir = dirs::home_dir()
            .unwrap()
            .join("Library/Logs/TimeTracker");
        std::fs::create_dir_all(&log_dir)?;

        // 写入 plist 文件
        std::fs::write(&self.launch_agent_path, plist_content)?;

        // 设置正确的权限
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let permissions = std::fs::Permissions::from_mode(0o644);
            std::fs::set_permissions(&self.launch_agent_path, permissions)?;
        }

        Ok(())
    }

    fn remove_launch_agent(&self) -> AppResult<()> {
        if self.launch_agent_path.exists() {
            std::fs::remove_file(&self.launch_agent_path)?;
        }
        Ok(())
    }

    fn get_frontmost_application_info(&self) -> AppResult<WindowInfo> {
        unsafe {
            let window_list = CGWindowListCopyWindowInfo(
                CGWindowListOption::OptionOnScreenOnly,
                kCGNullWindowID,
            );
            
            if window_list.is_null() {
                return Err(AppError::Platform("Failed to get window list".into()));
            }

            let count = CFArrayGetCount(window_list);
            let mut frontmost_window: Option<WindowInfo> = None;
            let mut lowest_layer = i32::MAX;

            for i in 0..count {
                let window_info = CFArrayGetValueAtIndex(window_list, i) as CFDictionaryRef;
                
                // 获取窗口层级
                let layer_value = CFDictionaryGetValue(
                    window_info,
                    Self::cf_string(kCGWindowLayer).cast(),
                ) as CFNumberRef;
                
                let mut layer: i32 = 0;
                if !CFNumberGetValue(layer_value, 3, &mut layer as *mut i32) {
                    continue;
                }

                // 只处理可见的普通窗口（层级 0）
                if layer == 0 && layer < lowest_layer {
                    if let Some(window) = self.extract_window_info(window_info) {
                        frontmost_window = Some(window);
                        lowest_layer = layer;
                    }
                }
            }

            CFRelease(window_list as *mut CFType);

            frontmost_window.ok_or_else(|| AppError::Platform("No active window found".into()))
        }
    }

    unsafe fn extract_window_info(&self, window_info: CFDictionaryRef) -> Option<WindowInfo> {
        let mut window = WindowInfo {
            title: String::new(),
            process_name: String::new(),
            process_id: 0,
            app_name: String::new(),
            window_title: String::new(),
        };

        // 获取进程 ID
        let pid_value = CFDictionaryGetValue(
            window_info,
            Self::cf_string(kCGWindowOwnerPID).cast(),
        ) as CFNumberRef;
        
        let mut pid: i32 = 0;
        if CFNumberGetValue(pid_value, 3, &mut pid as *mut i32) {
            window.process_id = pid as u32;
        }

        // 获取窗口标题
        if let Some(title) = self.get_string_value(window_info, kCGWindowName) {
            window.title = title;
            window.window_title = title;
        }

        // 获取应用名称
        if let Some(name) = self.get_string_value(window_info, kCGWindowOwnerName) {
            window.process_name = name.clone();
            window.app_name = name;
        }

        Some(window)
    }

    unsafe fn get_string_value(&self, dict: CFDictionaryRef, key: &str) -> Option<String> {
        let value = CFDictionaryGetValue(dict, Self::cf_string(key).cast()) as CFStringRef;
        if value.is_null() {
            return None;
        }

        let mut buffer = [0u8; 1024];
        if CFStringGetCString(
            value,
            buffer.as_mut_ptr() as *mut i8,
            buffer.len() as _,
            core_foundation::string::kCFStringEncodingUTF8,
        ) == 0 {
            return None;
        }

        CStr::from_ptr(buffer.as_ptr() as *const i8)
            .to_string_lossy()
            .into_owned()
            .into()
    }

    unsafe fn get_number_value(&self, dict: CFDictionaryRef, key: &str) -> Option<f64> {
        let value = CFDictionaryGetValue(dict, Self::cf_string(key).cast()) as CFNumberRef;
        if value.is_null() {
            return None;
        }

        let mut number = 0.0;
        if CFNumberGetValue(value, 12, &mut number as *mut f64) {
            Some(number)
        } else {
            None
        }
    }

    unsafe fn cf_string(s: &str) -> CFStringRef {
        core_foundation::string::CFString::new(s).as_concrete_TypeRef()
    }

    unsafe fn init_notification_center(&mut self) -> AppResult<()> {
        if self.notification_center.is_none() {
            let ns_user_notification_center = Class::get("NSUserNotificationCenter").ok_or_else(|| {
                AppError::Platform("Failed to get NSUserNotificationCenter class".into())
            })?;

            let center: *mut Object = msg_send![ns_user_notification_center, defaultUserNotificationCenter];
            if center.is_null() {
                return Err(AppError::Platform("Failed to get default notification center".into()));
            }

            self.notification_center = Some(center);
        }
        Ok(())
    }

    unsafe fn create_notification(&self, options: &NotificationOptions) -> AppResult<*mut Object> {
        let ns_user_notification = Class::get("NSUserNotification").ok_or_else(|| {
            AppError::Platform("Failed to get NSUserNotification class".into())
        })?;

        let notification: *mut Object = msg_send![ns_user_notification, new];
        if notification.is_null() {
            return Err(AppError::Platform("Failed to create notification".into()));
        }

        let title = core_foundation::string::CFString::new(&options.title);
        let message = core_foundation::string::CFString::new(&options.message);

        let _: () = msg_send![notification, setTitle:title];
        let _: () = msg_send![notification, setInformativeText:message];

        if options.sound {
            let _: () = msg_send![notification, setSoundName:"NSUserNotificationDefaultSoundName"];
        }

        if let Some(action) = &options.action_button {
            let action_str = core_foundation::string::CFString::new(action);
            let _: () = msg_send![notification, setActionButtonTitle:action_str];
        }

        if let Some(cancel) = &options.cancel_button {
            let cancel_str = core_foundation::string::CFString::new(cancel);
            let _: () = msg_send![notification, setOtherButtonTitle:cancel_str];
        }

        Ok(notification)
    }

    fn get_idle_time_impl(&self) -> AppResult<Duration> {
        unsafe {
            let session_dict = CGSessionCopyCurrentDictionary();
            if session_dict.is_null() {
                return Err(AppError::Platform("Failed to get session dictionary".into()));
            }

            let idle_time = self.get_number_value(session_dict, "CGSessionIdleTime")
                .unwrap_or(0.0);

            CFRelease(session_dict as *mut CFType);

            Ok(Duration::from_secs_f64(idle_time))
        }
    }
}

impl PlatformOperations for MacOSPlatform {
    fn get_active_window(&self) -> AppResult<WindowInfo> {
        self.get_frontmost_application_info()
    }

    fn set_autostart(&self, enabled: bool) -> AppResult<()> {
        if enabled {
            self.create_launch_agent()?;
        } else {
            self.remove_launch_agent()?;
        }
        Ok(())
    }

    fn is_autostart_enabled(&self) -> AppResult<bool> {
        Ok(self.launch_agent_path.exists())
    }

    fn set_dock_icon_visibility(&self, visibility: DockIconVisibility) -> AppResult<()> {
        unsafe {
            let ns_app = objc::runtime::Class::get("NSApplication")
                .ok_or_else(|| AppError::Platform("Failed to get NSApplication class".into()))?;
            
            let app: *mut Object = msg_send![ns_app, sharedApplication];
            let policy = match visibility {
                DockIconVisibility::Visible => 0, // NSApplicationActivationPolicyRegular
                DockIconVisibility::Hidden => 1,  // NSApplicationActivationPolicyAccessory
            };
            
            let _: () = msg_send![app, setActivationPolicy:policy];
            Ok(())
        }
    }

    fn get_dock_icon_visibility(&self) -> AppResult<DockIconVisibility> {
        unsafe {
            let ns_app = objc::runtime::Class::get("NSApplication")
                .ok_or_else(|| AppError::Platform("Failed to get NSApplication class".into()))?;
            
            let app: *mut Object = msg_send![ns_app, sharedApplication];
            let policy: i32 = msg_send![app, activationPolicy];
            
            Ok(match policy {
                0 => DockIconVisibility::Visible,
                _ => DockIconVisibility::Hidden,
            })
        }
    }

    fn bring_to_front(&self) -> AppResult<()> {
        unsafe {
            let ns_app = objc::runtime::Class::get("NSApplication")
                .ok_or_else(|| AppError::Platform("Failed to get NSApplication class".into()))?;
            
            let app: *mut Object = msg_send![ns_app, sharedApplication];
            let _: () = msg_send![app, activateIgnoringOtherApps:YES];
            Ok(())
        }
    }

    fn show_notification(&self, options: NotificationOptions) -> AppResult<()> {
        unsafe {
            self.init_notification_center()?;
            
            let notification = self.create_notification(&options)?;
            let center = self.notification_center.ok_or_else(|| {
                AppError::Platform("Notification center not initialized".into())
            })?;

            let _: () = msg_send![center, deliverNotification:notification];
            Ok(())
        }
    }

    fn request_notification_permissions(&self) -> AppResult<()> {
        unsafe {
            let result = CGRequestScreenCaptureAccess();
            if result == YES {
                Ok(())
            } else {
                Err(AppError::PermissionDenied("Screen capture access denied".into()))
            }
        }
    }

    fn get_system_idle_time(&self) -> AppResult<Duration> {
        self.get_idle_time_impl()
    }

    fn get_system_theme(&self) -> AppResult<String> {
        unsafe {
            let ns_app = objc::runtime::Class::get("NSApplication")
                .ok_or_else(|| AppError::Platform("Failed to get NSApplication class".into()))?;
            
            let app: *mut Object = msg_send![ns_app, sharedApplication];
            let appearance: *mut Object = msg_send![app, effectiveAppearance];
            let name: *mut Object = msg_send![appearance, name];
            
            if name.is_null() {
                return Ok("light".to_string());
            }

            let style: &str = if msg_send![name, isEqualToString:"NSAppearanceNameDarkAqua"] {
                "dark"
            } else {
                "light"
            };

            Ok(style.to_string())
        }
    }

    fn prevent_system_sleep(&self, prevent: bool) -> AppResult<()> {
        unsafe {
            let process_info = objc::runtime::Class::get("NSProcessInfo")
                .ok_or_else(|| AppError::Platform("Failed to get NSProcessInfo class".into()))?;
            
            let info: *mut Object = msg_send![process_info, processInfo];
            
            if prevent {
                let reason = core_foundation::string::CFString::new("TimeTracker active");
                let _: () = msg_send![info, beginActivityWithOptions:0x00FFFFFF reason:reason];
            } else {
                let _: () = msg_send![info, endActivity];
            }
            
            Ok(())
        }
    }
} 