//src/ui/views/pomodoro.rs

use crate::error::Result;
use crate::ui::{styles, components::*};
use crate::ui::TimeTrackerApp;
use eframe::egui;
use chrono::{Local, NaiveDateTime, Timelike, Datelike};
use std::time::Duration;

pub fn render(app: &mut TimeTrackerApp, ui: &mut egui::Ui) {
    ui.spacing_mut().item_spacing = egui::vec2(styles::SPACING_LARGE, styles::SPACING_LARGE);

    ui.horizontal(|ui| {
        // 主番茄钟区域
        ui.vertical(|ui| {
            render_pomodoro_timer(app, ui);
            ui.add_space(styles::SPACING_MEDIUM);
            render_pomodoro_controls(app, ui);
        });

        ui.separator();

        // 右侧统计区域
        ui.vertical(|ui| {
            render_pomodoro_stats(app, ui);
        });
    });
}

fn render_pomodoro_timer(app: &mut TimeTrackerApp, ui: &mut egui::Ui) {
    Card::new()
        .with_style(styles::CardStyle::elevated())
        .show(ui, |ui| {
            // 番茄钟状态显示
            let (status_text, color) = match app.pomodoro.get_state() {
                crate::pomodoro::PomodoroState::Working => (
                    "专注工作中",
                    styles::COLOR_PRIMARY,
                ),
                crate::pomodoro::PomodoroState::ShortBreak => (
                    "短休息",
                    styles::COLOR_SUCCESS,
                ),
                crate::pomodoro::PomodoroState::LongBreak => (
                    "长休息",
                    styles::COLOR_INFO,
                ),
                crate::pomodoro::PomodoroState::Paused(_) => (
                    "已暂停",
                    styles::COLOR_WARNING,
                ),
                _ => (
                    "准备开始",
                    styles::COLOR_TEXT_SECONDARY,
                ),
            };

            ui.vertical_centered(|ui| {
                ui.heading(status_text);
                ui.add_space(styles::SPACING_SMALL);

                // 显示剩余时间
                let remaining = app.pomodoro.get_remaining_time();
                ui.heading(
                    styles::format_text(
                        &format!("{:02}:{:02}", 
                            remaining.as_secs() / 60,
                            remaining.as_secs() % 60
                        ),
                        styles::heading(),
                        Some(color),
                    )
                );

                // 显示进度条
                let progress = app.pomodoro.get_progress();
                ui.add_space(styles::SPACING_SMALL);
                ProgressBar::new(progress)
                    .with_color(color)
                    .with_height(8.0)
                    .show(ui);
            });

            // 显示当前标签或项目（如果有）
            if let Some(project) = app.get_current_project() {
                ui.add_space(styles::SPACING_SMALL);
                ui.horizontal(|ui| {
                    ui.label("当前项目：");
                    Tag::new(&project.name)
                        .with_color(project.color)
                        .show(ui);
                });
            }
        });
}

fn render_pomodoro_controls(app: &mut TimeTrackerApp, ui: &mut egui::Ui) {
    ui.horizontal(|ui| {
        match app.pomodoro.get_state() {
            crate::pomodoro::PomodoroState::Idle => {
                if Button::new("开始专注")
                    .show(ui)
                    .clicked()
                {
                    app.pomodoro.start().ok();
                }
            }
            crate::pomodoro::PomodoroState::Working |
            crate::pomodoro::PomodoroState::ShortBreak |
            crate::pomodoro::PomodoroState::LongBreak => {
                if Button::new("暂停")
                    .with_style(styles::ButtonStyle::outlined())
                    .show(ui)
                    .clicked()
                {
                    app.pomodoro.pause().ok();
                }
                if Button::new("停止")
                    .with_style(styles::ButtonStyle::danger())
                    .show(ui)
                    .clicked()
                {
                    app.pomodoro.stop().ok();
                }
            }
            crate::pomodoro::PomodoroState::Paused(_) => {
                if Button::new("继续")
                    .show(ui)
                    .clicked()
                {
                    app.pomodoro.start().ok();
                }
                if Button::new("停止")
                    .with_style(styles::ButtonStyle::danger())
                    .show(ui)
                    .clicked()
                {
                    app.pomodoro.stop().ok();
                }
            }
        }
    });

    // 添加笔记
    if app.pomodoro.get_state() != crate::pomodoro::PomodoroState::Idle {
        ui.add_space(styles::SPACING_SMALL);
        ui.label("添加笔记：");
        if let Some(note) = &mut app.current_note {
            ui.text_edit_multiline(note);
        }
    }
}

fn render_pomodoro_stats(app: &mut TimeTrackerApp, ui: &mut egui::Ui) {
    ui.heading("今日统计");

    if let Ok(stats) = app.pomodoro.get_stats() {
        ui.horizontal(|ui| {
            Card::new()
                .show(ui, |ui| {
                    ui.label("完成番茄数");
                    ui.heading(format!("{}", stats.completed));
                });

            Card::new()
                .show(ui, |ui| {
                    ui.label("专注时间");
                    ui.heading(format_duration(stats.total_work_time));
                });

            Card::new()
                .show(ui, |ui| {
                    ui.label("最长专注");
                    ui.heading(format_duration(stats.longest_focus));
                });
        });

        // 显示完成记录
        ui.add_space(styles::SPACING_MEDIUM);
        ui.heading("完成记录");

        let mut records = stats.records;
        records.sort_by(|a, b| b.end_time.cmp(&a.end_time));

        egui::ScrollArea::vertical()
            .max_height(300.0)
            .show(ui, |ui| {
                for record in records {
                    Card::new()
                        .show(ui, |ui| {
                            ui.horizontal(|ui| {
                                let time_str = record.end_time.format("%H:%M").to_string();
                                match record.status {
                                    crate::pomodoro::PomodoroStatus::Completed => {
                                        ui.colored_label(
                                            styles::COLOR_SUCCESS,
                                            format!("✓ {}", time_str),
                                        );
                                    }
                                    crate::pomodoro::PomodoroStatus::Interrupted => {
                                        ui.colored_label(
                                            styles::COLOR_ERROR,
                                            format!("✗ {}", time_str),
                                        );
                                    }
                                }

                                if let Some(project) = record.project {
                                    Tag::new(&project)
                                        .with_color(styles::COLOR_PRIMARY)
                                        .show(ui);
                                }

                                // 显示标签
                                for tag in &record.tags {
                                    Tag::new(tag)
                                        .with_color(styles::COLOR_SECONDARY)
                                        .show(ui);
                                }
                            });

                            // 显示笔记
                            if let Some(note) = record.notes {
                                ui.add_space(styles::SPACING_SMALL);
                                ui.label(styles::format_text(
                                    &note,
                                    styles::small(),
                                    Some(styles::COLOR_TEXT_SECONDARY),
                                ));
                            }
                        });
                }
            });

        // 显示趋势图
        ui.add_space(styles::SPACING_MEDIUM);
        ui.heading("专注趋势");
        
        let chart_data = stats.daily_completed
            .iter()
            .map(|(date, count)| (*date.hour() as f64, *count as f64))
            .collect::<Vec<_>>();

        Chart::new(chart_data)
            .with_size(ui.available_width(), 150.0)
            .with_color(styles::COLOR_PRIMARY)
            .show(ui);

        // 显示建议
        if let Some(suggestion) = get_productivity_suggestion(&stats) {
            ui.add_space(styles::SPACING_MEDIUM);
            Card::new()
                .with_style(styles::CardStyle::default())
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        ui.label("💡");
                        ui.colored_label(
                            styles::COLOR_INFO,
                            suggestion,
                        );
                    });
                });
        }
    }
}

fn format_duration(duration: Duration) -> String {
    let hours = duration.as_secs() / 3600;
    let minutes = (duration.as_secs() % 3600) / 60;
    if hours > 0 {
        format!("{}h {}m", hours, minutes)
    } else {
        format!("{}m", minutes)
    }
}

fn get_productivity_suggestion(stats: &PomodoroStats) -> Option<String> {
    // 分析数据，生成建议
    if stats.completed == 0 {
        Some("开始你的第一个番茄钟吧！".to_string())
    } else {
        let completion_rate = stats.completed as f32 / (stats.completed + stats.interrupted) as f32;
        
        if completion_rate < 0.5 {
            Some("提示：尝试将手机调至勿扰模式，减少干扰。".to_string())
        } else if let Some(best_hour) = find_most_productive_hour(stats) {
            Some(format!(
                "你在{}点的专注效果最好，建议安排重要工作在这个时间段。",
                best_hour
            ))
        } else {
            None
        }
    }
}

fn find_most_productive_hour(stats: &PomodoroStats) -> Option<u32> {
    let mut hourly_completion = vec![0; 24];
    for (date, count) in &stats.daily_completed {
        hourly_completion[date.hour() as usize] += count;
    }
    
    hourly_completion.iter()
        .enumerate()
        .max_by_key(|(_, &count)| count)
        .map(|(hour, _)| hour as u32)
}

// 自定义统计数据结构
#[derive(Default)]
struct PomodoroStats {
    completed: u32,
    interrupted: u32,
    total_work_time: Duration,
    longest_focus: Duration,
    daily_completed: Vec<(NaiveDateTime, u32)>,
    records: Vec<PomodoroRecord>,
}

struct PomodoroRecord {
    end_time: NaiveDateTime,
    status: crate::pomodoro::PomodoroStatus,
    project: Option<String>,
    tags: Vec<String>,
    notes: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use eframe::egui::Context;

    #[test]
    fn test_pomodoro_rendering() {
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
        assert_eq!(format_duration(Duration::from_secs(3600)), "1h 0m");
        assert_eq!(format_duration(Duration::from_secs(90)), "1m");
        assert_eq!(format_duration(Duration::from_secs(5400)), "1h 30m");
    }

    #[test]
    fn test_productivity_suggestions() {
        let mut stats = PomodoroStats::default();
        
        // 测试新用户建议
        assert_eq!(
            get_productivity_suggestion(&stats),
            Some("开始你的第一个番茄钟吧！".to_string())
        );

        // 测试低完成率建议
        stats.completed = 1;
        stats.interrupted = 3;
        assert_eq!(
            get_productivity_suggestion(&stats),
            Some("提示：尝试将手机调至勿扰模式，减少干扰。".to_string())
        );
    }

    #[test]
    fn test_most_productive_hour() {
        let mut stats = PomodoroStats::default();
        
        // 添加测试数据
        stats.daily_completed.push((
            NaiveDateTime::from_timestamp_opt(0, 0).unwrap()
                .with_hour(9).unwrap(),
            3
        ));
        stats.daily_completed.push((
            NaiveDateTime::from_timestamp_opt(0, 0).unwrap()
                .with_hour(9).unwrap(),
            2
        ));
        stats.daily_completed.push((
            NaiveDateTime::from_timestamp_opt(0, 0).unwrap()
                .with_hour(14).unwrap(),
            1
        ));

        assert_eq!(find_most_productive_hour(&stats), Some(9));
    }
}

                                        