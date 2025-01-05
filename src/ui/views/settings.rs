//src/ui/views/settings.rs

use eframe::egui;
use crate::ui::{TimeTrackerApp, styles};
use super::components::Button;
use std::time::Duration;
use std::sync::Arc;
use parking_lot::Mutex;

pub fn render(app: &mut TimeTrackerApp, ui: &mut egui::Ui) {
    ui.heading("设置");
    ui.separator();

    let mut config = app.get_config();

    egui::ScrollArea::vertical()
        .auto_shrink([false; 2])
        .show(ui, |ui| {
            // 常规设置
            ui.heading("常规设置");
            ui.checkbox(&mut config.general.autostart, "开机自启动");
            ui.checkbox(&mut config.general.minimize_to_tray, "最小化到托盘");

            ui.separator();

            // 番茄钟设置
            render_pomodoro_settings(&mut *config, ui);

            ui.separator();

            // 备份设置
            ui.heading("备份设置");
            ui.checkbox(&mut config.storage.backup_enabled, "启用自动备份");

            ui.separator();

            // 保存按钮
            if Button::new("保存设置")
                .with_style(styles::ButtonStyle::primary())
                .show(ui)
                .clicked()
            {
                drop(config); // 先释放锁
                if let Err(e) = app.save_config() {
                    app.show_error(format!("保存设置失败: {}", e));
                }
            }
        });
}

fn render_pomodoro_settings(config: &mut Config, ui: &mut egui::Ui) {
    ui.heading("番茄钟设置");
    
    let mut work_minutes = config.pomodoro.work_duration.as_secs() / 60;
    ui.horizontal(|ui| {
        ui.label("工作时长(分钟)");
        if ui.add(egui::DragValue::new(&mut work_minutes)
            .clamp_range(1..=120))
            .changed() 
        {
            config.pomodoro.work_duration = Duration::from_secs(work_minutes * 60);
        }
    });

    let mut short_break_minutes = config.pomodoro.short_break_duration.as_secs() / 60;
    ui.horizontal(|ui| {
        ui.label("短休息时长(分钟)");
        if ui.add(egui::DragValue::new(&mut short_break_minutes)
            .clamp_range(1..=30))
            .changed()
        {
            config.pomodoro.short_break_duration = Duration::from_secs(short_break_minutes * 60);
        }
    });
}