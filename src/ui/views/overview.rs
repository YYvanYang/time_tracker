//src/ui/views/overview.rs

use eframe::egui;
use crate::ui::{TimeTrackerApp, styles};
use super::components::{Card, ProgressBar};

pub fn render(_app: &mut TimeTrackerApp, ui: &mut egui::Ui) {
    ui.heading("今日概览");
    ui.separator();

    // 显示当前状态
    Card::new()
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.vertical(|ui| {
                    ui.heading("工作中");
                    ui.label("剩余时间: 15:32");
                });

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ProgressBar::new(0.35).show(ui);
                });
            });
        });

    ui.separator();

    // 显示今日统计
    ui.horizontal(|ui| {
        Card::new()
            .show(ui, |ui| {
                ui.label("完成番茄数");
                ui.heading("6");
            });

        Card::new()
            .show(ui, |ui| {
                ui.label("专注时长");
                ui.heading("3小时12分");
            });

        Card::new()
            .show(ui, |ui| {
                ui.label("效率得分");
                ui.heading("78%");
            });
    });

    ui.separator();

    // 显示今日任务
    ui.heading("今日任务");
    egui::ScrollArea::vertical()
        .auto_shrink([false; 2])
        .show(ui, |_ui| {
            // TODO: 显示任务列表
        });
}