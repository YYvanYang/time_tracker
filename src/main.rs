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

#[tokio::main]
async fn main() -> Result<()> {
    // 初始化日志系统
    let log_path = logging::get_default_log_path();
    Logger::init(Some(log_path))
        .map_err(|e| TimeTrackerError::Platform(e.to_string()))?;

    log::info!("Starting Time Tracker application");

    // 加载配置
    let config = match Config::load() {
        Ok(config) => {
            log::info!("Configuration loaded successfully");
            config
        }
        Err(err) => {
            log::warn!("Failed to load config: {}, using defaults", err);
            Config::default()
        }
    };

    // 创建应用实例
    let app = ui::TimeTrackerApp::new(config);

    // 运行应用
    let native_options = eframe::NativeOptions {
        initial_window_size: Some(egui::vec2(
            app.config.ui.window_width as f32,
            app.config.ui.window_height as f32
        )),
        resizable: true,
        vsync: true,
        multisampling: 0,
        depth_buffer: 0,
        ..Default::default()
    };

    log::info!("Starting GUI");
    eframe::run_native(
        "Time Tracker",
        native_options,
        Box::new(|cc| {
            // 设置自定义字体
            setup_custom_fonts(&cc.egui_ctx);
            Box::new(app)
        }),
    )
    .map_err(|e| TimeTrackerError::Gui(e.to_string()))?;

    log::info!("Application terminated normally");
    Ok(())
}

fn setup_custom_fonts(ctx: &egui::Context) {
    let mut fonts = egui::FontDefinitions::default();
    
    // 添加自定义字体（这里使用系统字体，实际应用中可以打包字体文件）
    fonts.font_data.insert(
        "main_font".to_owned(),
        egui::FontData::from_static(include_bytes!("../assets/fonts/OpenSans-Regular.ttf")),
    );

    // 设置字体
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