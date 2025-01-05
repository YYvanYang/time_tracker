//src/ui/views/overview.rs

use crate::error::Result;
use crate::ui::{styles, components::*};
use crate::ui::TimeTrackerApp;
use eframe::egui;
use chrono::Local;
use std::time::Duration;

pub fn render(app: &mut TimeTrackerApp, ui: &mut egui::Ui) {
    ui.spacing_mut().item_spacing = egui::vec2(styles::SPACING_LARGE, styles::SPACING_LARGE);

    // 显示当前状态
    show_current_status(app, ui);

    // 今日统计
    show_today_stats(app, ui);

    // 当前项目和任务
    show_current_project(app, ui);

    // 时间分布图
    show_time_distribution(app, ui);
}

fn show_current_status(app: &mut TimeTrackerApp, ui: &mut egui::Ui) {
    Card::new()
        .with_style(styles::CardStyle::elevated())
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                match app.pomodoro.get_state() {
                    crate::pomodoro::PomodoroState::Working => {
                        ui.heading("工作中");
                        let remaining = app.pomodoro.get_remaining_time();
                        let progress = app.pomodoro.get_progress();
                        
                        ui.add_space(styles::SPACING_MEDIUM);
                        ProgressBar::new(progress)
                            .with_color(styles::COLOR_PRIMARY)
                            .show(ui);
                            
                        ui.label(format!(
                            "剩余 {}:{:02}",
                            remaining.as_secs() / 60,
                            remaining.as_secs() % 60
                        ));

                        if Button::new("暂停")
                            .with_style(styles::ButtonStyle::outlined())
                            .show(ui)
                            .clicked()
                        {
                            app.pomodoro.pause().ok();
                        }
                    }
                    crate::pomodoro::PomodoroState::ShortBreak |
                    crate::pomodoro::PomodoroState::LongBreak => {
                        ui.heading("休息中");
                        let remaining = app.pomodoro.get_remaining_time();
                        let progress = app.pomodoro.get_progress();

                        ui.add_space(styles::SPACING_MEDIUM);
                        ProgressBar::new(progress)
                            .with_color(styles::COLOR_SUCCESS)
                            .show(ui);

                        ui.label(format!(
                            "剩余 {}:{:02}",
                            remaining.as_secs() / 60,
                            remaining.as_secs() % 60
                        ));

                        if Button::new("跳过")
                            .with_style(styles::ButtonStyle::outlined())
                            .show(ui)
                            .clicked()
                        {
                            app.pomodoro.stop().ok();
                        }
                    }
                    _ => {
                        ui.heading("准备就绪");
                        if Button::new("开始工作")
                            .show(ui)
                            .clicked()
                        {
                            app.pomodoro.start().ok();
                        }
                    }
                }
            });
        });
}

fn show_today_stats(app: &mut TimeTrackerApp, ui: &mut egui::Ui) {
    let today = Local::now().date_naive();
    let daily_stats = app.storage.get_daily_stats(today).unwrap_or_default();

    ui.heading("今日统计");

    ui.horizontal(|ui| {
        // 工作时长
        Card::new()
            .show(ui, |ui| {
                ui.label(styles::format_text("工作时长", styles::body(), None));
                ui.heading(format_duration(daily_stats.total_work_time));
            });

        // 专注时长
        Card::new()
            .show(ui, |ui| {
                ui.label(styles::format_text("专注时长", styles::body(), None));
                ui.heading(format_duration(daily_stats.productive_time));
            });

        // 完成番茄数
        Card::new()
            .show(ui, |ui| {
                ui.label(styles::format_text("完成番茄数", styles::body(), None));
                ui.heading(format!("{}", daily_stats.completed_pomodoros));
            });
    });
}

fn show_current_project(app: &mut TimeTrackerApp, ui: &mut egui::Ui) {
    ui.heading("当前项目");

    if let Some(project) = app.get_current_project() {
        Card::new()
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.heading(&project.name);
                    if let Some(description) = &project.description {
                        ui.label(description);
                    }
                });

                // 显示项目进度
                if let Ok(stats) = app.storage.get_project_stats(&project) {
                    let progress = stats.completed_pomodoros as f32 / 
                        project.target_pomodoros as f32;
                    
                    ui.add_space(styles::SPACING_SMALL);
                    ProgressBar::new(progress)
                        .show_percentage(true)
                        .with_color(project.color)
                        .show(ui);
                }

                ui.horizontal(|ui| {
                    for tag in &project.tags {
                        Tag::new(tag)
                            .with_color(project.color)
                            .show(ui);
                    }
                });
            });
    } else {
        ui.label("没有进行中的项目");
        if Button::new("创建项目")
            .show(ui)
            .clicked()
        {
            app.push_dialog(Dialog::AddProject(ProjectDialog::new("新建项目")));
        }
    }
}

fn show_time_distribution(app: &mut TimeTrackerApp, ui: &mut egui::Ui) {
    ui.heading("时间分布");

    let today = Local::now().date_naive();
    if let Ok(distribution) = app.storage.get_time_distribution(today) {
        Chart::new(distribution)
            .with_size(ui.available_width(), 200.0)
            .show(ui);
    }
}

fn format_duration(duration: Duration) -> String {
    let hours = duration.as_secs() / 3600;
    let minutes = (duration.as_secs() % 3600) / 60;
    if hours > 0 {
        format!("{}小时{}分钟", hours, minutes)
    } else {
        format!("{}分钟", minutes)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use eframe::egui::Context;

    #[test]
    fn test_overview_rendering() {
        let ctx = Context::default();
        let mut app = TimeTrackerApp::test_new();

        ctx.run(|ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                render(&mut app, ui);
            });
        });
    }

    #[test]
    fn test_duration_formatting() {
        assert_eq!(format_duration(Duration::from_secs(3600)), "1小时0分钟");
        assert_eq!(format_duration(Duration::from_secs(90)), "1分钟");
        assert_eq!(format_duration(Duration::from_secs(5400)), "1小时30分钟");
    }
}