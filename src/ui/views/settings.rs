//src/ui/views/settings.rs

use eframe::egui;
use crate::ui::{TimeTrackerApp, styles};
use super::components::Button;
use std::time::Duration;

pub fn render(app: &mut TimeTrackerApp, ui: &mut egui::Ui) {
    ui.heading("设置");
    ui.separator();

    let mut config = app.get_config();
    
    // ... 设置界面的实现
}