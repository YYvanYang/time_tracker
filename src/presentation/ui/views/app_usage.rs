//src/ui/views/app_usage.rs

use eframe::egui;
use crate::ui::{TimeTrackerApp, styles};
use super::components::{Card, Chart};
use chrono::{Local, Duration, NaiveDateTime};

pub fn render(app: &mut TimeTrackerApp, ui: &mut egui::Ui) {
    ui.heading("应用使用统计");
    ui.separator();

    ui.horizontal(|ui| {
        // 时间范围选择
        ui.label("时间范围:");
        
        let time_ranges = ["今天", "昨天", "本周", "上周", "本月"];
        egui::ComboBox::from_label("")
            .selected_text(time_ranges[app.selected_time_range])
            .show_ui(ui, |ui| {
                for (i, range) in time_ranges.iter().enumerate() {
                    ui.selectable_value(&mut app.selected_time_range, i, *range);
                }
            });
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
        .show(ui, |ui| {
            for app_data in &app.usage_data {
                ui.horizontal(|ui| {
                    ui.label(&app_data.name);
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.label(&format!("{:.1}小时", app_data.duration.num_minutes() as f32 / 60.0));
                        
                        // 显示进度条
                        let progress = app_data.duration.num_minutes() as f32 / (8.0 * 60.0); // 假设以8小时为基准
                        let rect = ui.available_rect_before_wrap();
                        let bar_width = 200.0;
                        let bar_height = 8.0;
                        let bar_rect = egui::Rect::from_min_size(
                            egui::pos2(rect.right() - bar_width - 10.0, rect.center().y - bar_height/2.0),
                            egui::vec2(bar_width, bar_height),
                        );
                        
                        ui.painter().rect_filled(
                            bar_rect,
                            2.0,
                            styles::GRAY_COLOR,
                        );
                        
                        ui.painter().rect_filled(
                            egui::Rect::from_min_size(
                                bar_rect.min,
                                egui::vec2(bar_width * progress.min(1.0), bar_height),
                            ),
                            2.0,
                            styles::PRIMARY_COLOR,
                        );
                    });
                });
                ui.separator();
            }
        });
}