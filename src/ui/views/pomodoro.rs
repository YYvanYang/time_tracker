//src/ui/views/pomodoro.rs

use crate::error::Result;
use crate::ui::{styles, components::*};
use crate::ui::TimeTrackerApp;
use crate::storage::{PomodoroStatus, Tag};
use eframe::egui;
use chrono::{Local, NaiveDateTime, Timelike};
use std::time::Duration;
use crate::pomodoro::PomodoroStats as CorePomodoroStats;

pub fn render(app: &mut TimeTrackerApp, ui: &mut egui::Ui) {
    Card::new()
        .with_style(styles::CardStyle::elevated())
        .show(ui, |ui| {
            let (status_text, color) = match app.pomodoro_timer.lock().unwrap().get_state() {
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
                let remaining = app.pomodoro_timer.lock().unwrap().get_remaining_time();
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
                let progress = app.pomodoro_timer.lock().unwrap().get_progress();
                ui.add_space(styles::SPACING_SMALL);
                ProgressBar::new(progress)
                    .with_color(color)
                    .with_height(8.0)
                    .show(ui);
            });

            // 显示当前标签或项目
            if let Some(project) = app.get_current_project() {
                ui.add_space(styles::SPACING_SMALL);
                ui.horizontal(|ui| {
                    ui.label("当前项目：");
                    Tag::new(&project.name)
                        .with_color(project.color.as_ref().map_or_else(
                            || "#FFFFFF".to_string(),
                            |c| c.clone()
                        ))
                        .show(ui);
                });
            }
        });
}

pub fn render_pomodoro_controls(app: &mut TimeTrackerApp, ui: &mut egui::Ui) {
    ui.horizontal(|ui| {
        match app.pomodoro_timer.lock().unwrap().get_state() {
            crate::pomodoro::PomodoroState::Idle => {
                if Button::new("开始专注")
                    .show(ui)
                    .clicked()
                {
                    app.pomodoro_timer.lock().unwrap().start().ok();
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
                    app.pomodoro_timer.lock().unwrap().pause().ok();
                }
                if Button::new("停止")
                    .with_style(styles::ButtonStyle::danger())
                    .show(ui)
                    .clicked()
                {
                    app.pomodoro_timer.lock().unwrap().stop().ok();
                }
            }
            crate::pomodoro::PomodoroState::Paused(_) => {
                if Button::new("继续")
                    .show(ui)
                    .clicked()
                {
                    app.pomodoro_timer.lock().unwrap().start().ok();
                }
                if Button::new("停止")
                    .with_style(styles::ButtonStyle::danger())
                    .show(ui)
                    .clicked()
                {
                    app.pomodoro_timer.lock().unwrap().stop().ok();
                }
            }
        }
    });

    // 添加笔记
    if let Some(note) = &mut app.ui_state.current_note {
        ui.text_edit_multiline(note);
    }
}

fn render_pomodoro_stats(app: &mut TimeTrackerApp, ui: &mut egui::Ui) {
    if let Ok(stats) = app.pomodoro_timer.lock().unwrap().get_stats() {
        Card::new()
            .with_style(styles::CardStyle::elevated())
            .show(ui, |ui| {
                ui.heading("统计");
                ui.add_space(styles::SPACING_SMALL);

                // 显示完成数量
                ui.heading(format!("{}", stats.total_completed));
                ui.label("已完成番茄钟");

                ui.add_space(styles::SPACING_MEDIUM);

                // 显示专注时长
                ui.heading(format_duration(stats.total_work_time));
                ui.label("总专注时长");

                // 显示最长专注时间
                ui.heading(format_duration(Duration::from_secs(stats.longest_streak as u64)));
                ui.label("最长专注时间");

                // 显示历史记录
                ui.add_space(styles::SPACING_LARGE);
                ui.heading("历史记录");
                
                let daily_records = stats.daily_completed.iter()
                    .map(|(date, count)| (date.hour() as f64, *count as f64))
                    .collect::<Vec<_>>();

                Chart::new(daily_records)
                    .with_size(ui.available_width(), 150.0)
                    .with_color(styles::COLOR_PRIMARY)
                    .show(ui);
            });
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

fn get_productivity_suggestion(stats: &CorePomodoroStats) -> Option<String> {
    if stats.total_completed == 0 {
        Some("开始你的第一个番茄钟吧！".to_string())
    } else {
        let completion_rate = stats.total_completed as f32 / 
            (stats.total_completed + stats.total_interrupted) as f32;
        
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

fn find_most_productive_hour(stats: &CorePomodoroStats) -> Option<u32> {
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

#[derive(Debug)]
struct PomodoroRecord {
    end_time: NaiveDateTime,
    status: PomodoroStatus,
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

                                        