//src/ui/views/app_usage.rs

use eframe::egui;
use crate::ui::{TimeTrackerApp, styles};
use super::components::{Card, Chart};
use chrono::{Local, Duration};

pub fn render(_app: &mut TimeTrackerApp, ui: &mut egui::Ui) {
    ui.heading("应用使用统计");
    ui.separator();

    ui.horizontal(|ui| {
        // 时间范围选择
        ui.label("时间范围:");
        // TODO: 添加日期选择器
    });

    // 显示总览数据
    ui.horizontal(|ui| {
        Card::new()
            .show(ui, |ui| {
                ui.label("总使用时长");
                ui.heading("8小时32分钟");
            });

        Card::new()
            .show(ui, |ui| {
                ui.label("生产力得分");
                ui.heading("76%");
            });

        Card::new()
            .show(ui, |ui| {
                ui.label("最常用应用");
                ui.heading("Visual Studio Code");
            });
    });

    ui.separator();

    // 显示使用时长图表
    Chart::new(vec![(0.0, 1.0), (1.0, 2.0), (2.0, 1.5)])
        .with_size(ui.available_width(), 200.0)
        .show(ui);

    ui.separator();

    // 显示应用列表
    egui::ScrollArea::vertical()
        .auto_shrink([false; 2])
        .show(ui, |_ui| {
            // TODO: 显示应用列表
        });
}