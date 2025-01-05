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

use crate::error::{Result, TimeTrackerError};
use crate::config::Config;
use crate::logging::Logger;
use crate::platform::PlatformInterface;
use crate::app_state::AppStateManager;
use crate::storage::Storage;
use crate::app_tracker::AppTracker;
use crate::pomodoro::PomodoroTimer;

#[tokio::main]
async fn main() -> Result<()> {
    // 初始化平台特定功能
    let platform = platform::init()?;

    // 加载配置
    let config = Config::load()?;
    
    // 初始化存储
    let storage = Storage::new(&config.storage)?;
    
    // 初始化应用追踪器
    let app_tracker = AppTracker::new()?;
    
    // 初始化番茄钟
    let pomodoro = PomodoroTimer::new(config.pomodoro.clone(), Default::default())?;
    
    // 初始化状态管理器
    let state_manager = AppStateManager::new(config.storage.data_dir.clone(), true)?;

    // 创建应用实例
    let app = ui::TimeTrackerApp::new(
        config,
        storage,
        app_tracker,
        pomodoro,
        state_manager,
    );

    // 运行应用
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([800.0, 600.0])
            .with_resizable(true),
        ..Default::default()
    };

    eframe::run_native(
        "Time Tracker",
        options,
        Box::new(|cc| {
            setup_custom_fonts(&cc.egui_ctx);
            Box::new(app)
        }),
    ).map_err(|e| TimeTrackerError::Gui(e.to_string()))?;

    Ok(())
}

fn setup_custom_fonts(ctx: &egui::Context) {
    let mut fonts = egui::FontDefinitions::default();
    
    // 使用系统默认字体
    fonts.font_data.insert(
        "main_font".to_owned(),
        egui::FontData::from_static(include_bytes!("../assets/fonts/OpenSans-Regular.ttf")), // 临时使用内置字体
    );

    fonts.families
        .get_mut(&egui::FontFamily::Proportional)
        .unwrap()
        .insert(0, "main_font".to_owned());

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