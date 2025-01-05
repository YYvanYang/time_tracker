//src/ui/views/app_usage.rs

use crate::ui::{styles, components::*};
use crate::ui::TimeTrackerApp;
use eframe::egui;
use chrono::{Local, Duration, Datelike};
use std::collections::HashMap;

pub fn render(app: &mut TimeTrackerApp, ui: &mut egui::Ui) {
    ui.spacing_mut().item_spacing = egui::vec2(styles::SPACING_LARGE, styles::SPACING_LARGE);

    // 时间范围选择
    static mut SELECTED_RANGE: TimeRange = TimeRange::Today;
    let selected_range = unsafe { &mut SELECTED_RANGE };

    ui.horizontal(|ui| {
        ui.heading("应用使用统计");
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            ui.horizontal(|ui| {
                if ui.selectable_value(selected_range, TimeRange::Today, "今天").clicked() ||
                   ui.selectable_value(selected_range, TimeRange::Week, "本周").clicked() ||
                   ui.selectable_value(selected_range, TimeRange::Month, "本月").clicked() {
                    // 切换时间范围时刷新数据
                }
            });
        });
    });

    let (start_time, end_time) = get_time_range(*selected_range);
    
    // 获取统计数据
    if let Ok(stats) = app.storage.get_app_usage_stats(start_time, end_time) {
        // 上方卡片显示总览
        ui.horizontal(|ui| {
            Card::new()
                .show(ui, |ui| {
                    ui.label(styles::format_text("总使用时间", styles::body(), None));
                    ui.heading(format_duration(stats.total_time));
                });

            Card::new()
                .show(ui, |ui| {
                    ui.label(styles::format_text("生产时间", styles::body(), None));
                    ui.heading(format_duration(stats.productive_time));
                });

            Card::new()
                .show(ui, |ui| {
                    ui.label(styles::format_text("生产力得分", styles::body(), None));
                    ui.heading(format!("{}%", (stats.productivity_ratio * 100.0) as i32));
                });
        });

        // 左侧应用列表，右侧详细信息
        ui.horizontal(|ui| {
            ui.vertical(|ui| {
                // 应用列表
                ui.heading("应用排行");
                for app_stat in &stats.app_stats {
                    if ui.selectable_label(
                        stats.selected_app.as_ref() == Some(&app_stat.name),
                        format!(
                            "{} - {}",
                            app_stat.name,
                            format_duration(app_stat.total_time)
                        )
                    ).clicked() {
                        // 选中应用，显示详情
                    }
                }
            }).min_width(250.0);

            ui.separator();

            ui.vertical(|ui| {
                // 应用详情
                if let Some(app_name) = &stats.selected_app {
                    if let Some(app_stat) = stats.app_stats.iter()
                        .find(|stat| &stat.name == app_name)
                    {
                        ui.heading(&app_stat.name);
                        
                        // 使用时间分布图
                        Chart::new(app_stat.hourly_usage.clone())
                            .with_size(ui.available_width(), 200.0)
                            .show(ui);

                        ui.add_space(styles::SPACING_MEDIUM);

                        // 详细统计信息
                        ui.horizontal(|ui| {
                            Card::new()
                                .show(ui, |ui| {
                                    ui.label("使用时长");
                                    ui.heading(format_duration(app_stat.total_time));
                                });

                            Card::new()
                                .show(ui, |ui| {
                                    ui.label("使用次数");
                                    ui.heading(format!("{}", app_stat.session_count));
                                });

                            Card::new()
                                .show(ui, |ui| {
                                    ui.label("平均时长");
                                    ui.heading(format_duration(app_stat.average_duration));
                                });
                        });

                        // 窗口标题记录
                        ui.collapsing("窗口记录", |ui| {
                            for (title, duration) in &app_stat.window_stats {
                                ui.label(format!(
                                    "{} - {}",
                                    title,
                                    format_duration(*duration)
                                ));
                            }
                        });

                        // 使用趋势图
                        ui.heading("使用趋势");
                        Chart::new(app_stat.daily_usage.clone())
                            .with_size(ui.available_width(), 150.0)
                            .show(ui);

                        // 生产力分析
                        if app_stat.is_productive {
                            ui.colored_label(
                                styles::COLOR_SUCCESS,
                                "✓ 这是一个生产力应用"
                            );
                        } else {
                            ui.colored_label(
                                styles::COLOR_WARNING,
                                "! 这可能是一个干扰应用"
                            );
                        }
                    }
                } else {
                    ui.colored_label(styles::COLOR_TEXT_SECONDARY, "选择应用查看详情");
                }
            });
        });
    }
}

#[derive(PartialEq, Copy, Clone)]
enum TimeRange {
    Today,
    Week,
    Month,
    Custom(chrono::DateTime<Local>, chrono::DateTime<Local>),
}

fn get_time_range(range: TimeRange) -> (chrono::DateTime<Local>, chrono::DateTime<Local>) {
    let now = Local::now();
    match range {
        TimeRange::Today => {
            let start = now.date_naive().and_hms_opt(0, 0, 0).unwrap()
                .and_local_timezone(Local).unwrap();
            (start, now)
        }
        TimeRange::Week => {
            let start = now.date_naive()
                .week(chrono::Weekday::Monday)
                .first_day()
                .and_hms_opt(0, 0, 0).unwrap()
                .and_local_timezone(Local).unwrap();
            (start, now)
        }
        TimeRange::Month => {
            let start = now.date_naive()
                .with_day(1).unwrap()
                .and_hms_opt(0, 0, 0).unwrap()
                .and_local_timezone(Local).unwrap();
            (start, now)
        }
        TimeRange::Custom(start, end) => (start, end),
    }
}

fn format_duration(duration: std::time::Duration) -> String {
    let total_minutes = duration.as_secs() / 60;
    let hours = total_minutes / 60;
    let minutes = total_minutes % 60;
    
    if hours > 0 {
        format!("{}h {}m", hours, minutes)
    } else {
        format!("{}m", minutes)
    }
}

#[derive(Default)]
struct AppStats {
    total_time: std::time::Duration,
    productive_time: std::time::Duration,
    productivity_ratio: f32,
    app_stats: Vec<AppStatistics>,
    selected_app: Option<String>,
}

#[derive(Clone)]
struct AppStatistics {
    name: String,
    total_time: std::time::Duration,
    session_count: u32,
    average_duration: std::time::Duration,
    is_productive: bool,
    window_stats: HashMap<String, std::time::Duration>,
    hourly_usage: Vec<(f64, f64)>,
    daily_usage: Vec<(f64, f64)>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use eframe::egui::Context;
    use std::time::Duration;

    #[test]
    fn test_time_range_calculation() {
        let now = Local::now();
        
        // 测试今天的时间范围
        let (start, end) = get_time_range(TimeRange::Today);
        assert_eq!(start.date_naive(), now.date_naive());
        assert_eq!(start.time().hour(), 0);
        assert_eq!(start.time().minute(), 0);
        
        // 测试本周的时间范围
        let (start, end) = get_time_range(TimeRange::Week);
        assert!(start <= now);
        assert_eq!(start.weekday(), chrono::Weekday::Mon);
        
        // 测试本月的时间范围
        let (start, end) = get_time_range(TimeRange::Month);
        assert_eq!(start.day(), 1);
        assert_eq!(start.month(), now.month());
    }

    #[test]
    fn test_duration_formatting() {
        assert_eq!(format_duration(Duration::from_secs(3600)), "1h 0m");
        assert_eq!(format_duration(Duration::from_secs(90)), "1m");
        assert_eq!(format_duration(Duration::from_secs(5400)), "1h 30m");
    }

    #[test]
    fn test_app_usage_rendering() {
        let ctx = Context::default();
        let mut app = TimeTrackerApp::test_new();

        ctx.run(|ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                render(&mut app, ui);
            });
        });
    }
}


            ui.separator();

            ui.vertical(|ui| {
                // 应用详情
                if let Some(app_name) = &stats.selected_app {
                    if let Some(app_stat) = stats.app_stats.iter()
                        .find(|stat| &stat.name == app_name)
                    {
                        ui.heading(&app_stat.name);
                        
                        // 使用时间分布图
                        Chart::new(app_stat.hourly_usage.clone())
                            .with_size(ui.available_width(), 200.0)
                            .show(ui);

                        ui.add_space(styles::SPACING_MEDIUM);

                        // 详细统计信息
                        ui.horizontal(|ui| {
                            Card::new()
                                .show(ui, |ui| {
                                    ui.label("使用时长");
                                    ui.heading(format_duration(app_stat.total_time));
                                });

                            Card::new()
                                .show(ui, |ui| {
                                    ui.label("使用次数");
                                    ui.heading(format!("{}", app_stat.session_count));
                                });

                            Card::new()
                                .show(ui, |ui| {
                                    ui.label("平均时长");
                                    ui.heading(format_duration(app_stat.average_duration));
                                });
                        });

                        // 窗口标题记录
                        ui.collapsing("窗口记录", |ui| {
                            for (title, duration) in &app_stat.window_stats {
                                ui.label(format!(
                                    "{} - {}",
                                    title,
                                    format_duration(*duration)
                                ));
                            }
                        });
                    }
                } else {
                    ui.colored_label(styles::COLOR_TEXT_SECONDARY, "选择应用查看详情");
                }
            });