//src/ui/views/statistics.rs

use eframe::egui;
use crate::ui::{TimeTrackerApp, styles};
use super::components::{Card, Chart};

pub fn render(_app: &mut TimeTrackerApp, ui: &mut egui::Ui) {
    ui.heading("统计分析");
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
                ui.label("完成番茄数");
                ui.heading("24");
            });

        Card::new()
            .show(ui, |ui| {
                ui.label("专注时长");
                ui.heading("12小时");
            });

        Card::new()
            .show(ui, |ui| {
                ui.label("生产力得分");
                ui.heading("85%");
            });
    });

    ui.separator();

    // 显示趋势图表
    Chart::new(vec![(0.0, 4.0), (1.0, 6.0), (2.0, 5.0)])
        .with_size(ui.available_width(), 200.0)
        .show(ui);

    ui.separator();

    // 显示详细统计
    egui::ScrollArea::vertical()
        .auto_shrink([false; 2])
        .show(ui, |_ui| {
            // TODO: 显示详细统计数据
        });
}