mod error;
mod config;
mod logging;
mod app_tracker;
mod pomodoro;
mod shutdown;
mod storage;
mod sync;
mod analysis;
mod cache;
mod visualization;
mod platform;
mod ui;
mod tray;
mod hotkeys;

use crate::error::{Result, TimeTrackerError};
use crate::config::Config;
use crate::logging::Logger;
use crate::storage::{Storage, StorageConfig};
use crate::app_tracker::{AppTracker, AppUsageConfig};
use crate::pomodoro::{PomodoroTimer, PomodoroConfig, PomodoroCallbacks};
use crate::tray::TrayManager;
use crate::hotkeys::{HotkeyManager, HotkeyConfig};
use crate::storage::app_state::AppStateManager;

use std::sync::Arc;
use tokio::sync::Mutex;
use eframe::{self, egui};
use egui::ViewportBuilder;
use std::sync::mpsc;

#[tokio::main]
async fn main() -> Result<()> {
    // 初始化日志
    Logger::init(None)?;

    // 设置全局错误处理器
    std::panic::set_hook(Box::new(|panic_info| {
        log::error!("Application panic: {:?}", panic_info);
    }));

    // 加载配置
    let config = Config::load()?;

    // 转换配置类型
    let storage_config = StorageConfig::from(config.storage.clone());
    let storage = Arc::new(std::sync::Mutex::new(Storage::new(storage_config)?));

    // 初始化应用状态管理器
    let app_state_manager = AppStateManager::new(config.storage.data_dir.clone(), true)?;
    let app_state_manager = Arc::new(std::sync::Mutex::new(app_state_manager));

    // 转换番茄钟配置
    let pomodoro_config = PomodoroConfig::from(config.pomodoro.clone());
    let callbacks = PomodoroCallbacks::default();
    let pomodoro_timer = PomodoroTimer::new(pomodoro_config, callbacks);
    let pomodoro_timer = Arc::new(std::sync::Mutex::new(pomodoro_timer));

    // 初始化应用追踪器
    let app_tracker_config = AppUsageConfig::default(); // 使用默认配置
    let app_tracker = AppTracker::new(app_tracker_config)?;
    let app_tracker = Arc::new(std::sync::Mutex::new(app_tracker));

    // 初始化系统托盘
    let (tray_sender, tray_receiver) = mpsc::channel();
    let icon_path = config.storage.data_dir.join("assets/icons/tray-icon.png");
    let tray_manager = TrayManager::new(icon_path, tray_sender)?;
    let tray_event_receiver = tray_manager.get_event_receiver();
    let tray_manager = Arc::new(std::sync::Mutex::new(tray_manager));

    // 初始化快捷键管理器
    let hotkey_config = HotkeyConfig::default();
    let hotkey_manager = Arc::new(std::sync::Mutex::new(HotkeyManager::new(hotkey_config)));

    // 初始化 GUI
    let native_options = eframe::NativeOptions {
        viewport: ViewportBuilder::default()
            .with_inner_size([800.0, 600.0])
            .with_min_inner_size([400.0, 300.0])
            .with_icon(load_icon()),
        ..Default::default()
    };

    let shared_storage = storage.clone();
    let shared_pomodoro_timer = pomodoro_timer.clone();
    let shared_app_tracker = app_tracker.clone();
    let shared_app_state_manager = app_state_manager.clone();
    let shared_tray_manager = tray_manager.clone();
    let shared_hotkey_manager = hotkey_manager.clone();

    // 将 config 包装在 Arc<Mutex<_>> 中
    let config = Arc::new(std::sync::Mutex::new(config));
    
    eframe::run_native(
        "Time Tracker",
        native_options,
        Box::new(move |cc| {
            setup_custom_fonts(&cc.egui_ctx);
            let app = ui::TimeTrackerApp::new(
                config.clone(),
                shared_storage,
                shared_pomodoro_timer,
                shared_app_tracker,
                shared_app_state_manager,
                shared_tray_manager,
                shared_hotkey_manager,
                tray_event_receiver,
            );
            Box::new(app)
        }),
    ).map_err(|e| TimeTrackerError::Gui(e.to_string()))?;

    Ok(())
}

fn setup_custom_fonts(ctx: &egui::Context) {
    let mut fonts = egui::FontDefinitions::default();

    // 根据不同操作系统配置推荐字体
    #[cfg(target_os = "windows")]
    {
        // Windows 优先使用微软雅黑
        if let Ok(font_data) = std::fs::read("C:\\Windows\\Fonts\\msyh.ttf") {
            fonts.font_data.insert(
                "microsoft_yahei".to_owned(),
                egui::FontData::from_owned(font_data),
            );
            fonts.families
                .get_mut(&egui::FontFamily::Proportional)
                .unwrap()
                .insert(0, "microsoft_yahei".to_owned());
        }
    }

    #[cfg(target_os = "macos")]
    {
        // macOS 优先使用苹方
        fonts.families
            .get_mut(&egui::FontFamily::Proportional)
            .unwrap()
            .insert(0, "PingFang SC".to_owned());
    }

    #[cfg(target_os = "linux")]
    {
        // Linux 优先使用文泉驿微米黑
        fonts.families
            .get_mut(&egui::FontFamily::Proportional)
            .unwrap()
            .extend_from_slice(&[
                "Noto Sans CJK SC".to_owned(),
                "WenQuanYi Micro Hei".to_owned(),
            ]);
    }

    // 设置后备字体
    fonts.families
        .get_mut(&egui::FontFamily::Proportional)
        .unwrap()
        .extend_from_slice(&[
            "Segoe UI".to_owned(),
            "Arial".to_owned(),
            "Helvetica".to_owned(),
        ]);

    ctx.set_fonts(fonts);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_loading() -> Result<()> {
        let config = Config::load()?;
        assert!(config.validate().is_ok());
        Ok(())
    }
}

fn load_icon() -> egui::IconData {
    // 创建一个简单的默认图标 (32x32 纯色图标)
    let width = 32;
    let height = 32;
    let mut rgba = Vec::with_capacity(width * height * 4);
    
    // 使用蓝色填充图标 (R=51, G=153, B=255, A=255)
    for _ in 0..(width * height) {
        rgba.extend_from_slice(&[51, 153, 255, 255]);
    }

    egui::IconData {
        rgba,
        width: width as u32,
        height: height as u32,
    }
}