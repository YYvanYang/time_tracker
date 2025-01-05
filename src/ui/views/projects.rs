use eframe::egui;
use crate::ui::{TimeTrackerApp, styles};
use super::components::{Button, Card};

pub fn render(_app: &mut TimeTrackerApp, ui: &mut egui::Ui) {
    ui.horizontal(|ui| {
        ui.heading("项目管理");
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            if Button::new("添加项目")
                .with_style(styles::ButtonStyle::primary())
                .show(ui)
                .clicked()
            {
                // TODO: 显示添加项目对话框
            }
        });
    });
    ui.separator();

    egui::ScrollArea::vertical()
        .auto_shrink([false; 2])
        .show(ui, |_ui| {
            // TODO: 显示项目列表
        });
} 