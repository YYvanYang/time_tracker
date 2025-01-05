use eframe::egui;
use crate::ui::TimeTrackerApp;
use crate::config::Theme;
use std::time::Duration;
use log;

pub fn render(app: &mut TimeTrackerApp, ui: &mut egui::Ui) {
    ui.heading("设置");
    ui.separator();

    let mut config = app.get_config();
    let mut config_changed = false;
    
    // 番茄钟设置
    ui.group(|ui| {
        ui.heading("番茄钟设置");
        
        // 工作时长
        ui.horizontal(|ui| {
            ui.label("工作时长 (分钟):");
            let mut work_duration = config.pomodoro.work_duration.as_secs() / 60;
            if ui.add(egui::DragValue::new(&mut work_duration)
                .clamp_range(1..=120)
                .speed(1.0))
                .changed() 
            {
                config.pomodoro.work_duration = Duration::from_secs(work_duration * 60);
                config_changed = true;
            }
        });

        // 短休息时长
        ui.horizontal(|ui| {
            ui.label("短休息时长 (分钟):");
            let mut break_duration = config.pomodoro.short_break_duration.as_secs() / 60;
            if ui.add(egui::DragValue::new(&mut break_duration)
                .clamp_range(1..=30)
                .speed(1.0))
                .changed() 
            {
                config.pomodoro.short_break_duration = Duration::from_secs(break_duration * 60);
                config_changed = true;
            }
        });

        // 长休息时长
        ui.horizontal(|ui| {
            ui.label("长休息时长 (分钟):");
            let mut long_break = config.pomodoro.long_break_duration.as_secs() / 60;
            if ui.add(egui::DragValue::new(&mut long_break)
                .clamp_range(1..=60)
                .speed(1.0))
                .changed() 
            {
                config.pomodoro.long_break_duration = Duration::from_secs(long_break * 60);
                config_changed = true;
            }
        });

        // 长休息间隔
        ui.horizontal(|ui| {
            ui.label("长休息间隔 (番茄钟数):");
            if ui.add(egui::DragValue::new(&mut config.pomodoro.long_break_interval)
                .clamp_range(1..=10)
                .speed(1.0))
                .changed() 
            {
                config_changed = true;
            }
        });
    });

    ui.add_space(16.0);

    // 界面设置
    ui.group(|ui| {
        ui.heading("界面设置");
        
        // 主题选择
        ui.horizontal(|ui| {
            ui.label("主题:");
            egui::ComboBox::from_id_source("theme_selector")
                .selected_text(format!("{:?}", config.ui.theme))
                .show_ui(ui, |ui| {
                    let themes = [Theme::Light, Theme::Dark, Theme::System];
                    for theme in themes {
                        if ui.selectable_label(config.ui.theme == theme, format!("{:?}", theme))
                            .clicked() 
                        {
                            config.ui.theme = theme;
                            config_changed = true;
                        }
                    }
                });
        });

        // 开机自启
        ui.horizontal(|ui| {
            if ui.checkbox(&mut config.general.autostart, "开机自启")
                .changed() 
            {
                config_changed = true;
            }
        });

        // 最小化到托盘
        ui.horizontal(|ui| {
            if ui.checkbox(&mut config.general.minimize_to_tray, "最小化到托盘")
                .changed() 
            {
                config_changed = true;
            }
        });
    });

    // 如果配置发生改变，保存配置
    if config_changed {
        let new_config = config.clone();  // 克隆到新变量
        drop(config);  // 释放原始配置的借用
        *app.config.lock().unwrap() = new_config;  // 使用新变量更新配置
        if let Err(e) = app.save_config() {
            log::error!("保存配置失败: {}", e);
        }
    }
}