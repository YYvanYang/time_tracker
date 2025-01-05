//src/ui/views/statistics.rs

use crate::ui::{styles, components::*};
use crate::ui::TimeTrackerApp;
use eframe::egui;
use chrono::{Local, Duration, Datelike};
use std::collections::HashMap;

pub fn render(app: &mut TimeTrackerApp, ui: &mut egui::Ui) {
    ui.spacing_mut().item_spacing = egui::vec2(styles::SPACING_LARGE, styles::SPACING_LARGE);

    // 时间范围选择
    static mut SELECTED_RANGE: TimeRange = TimeRange::Week;
    let selected_range = unsafe { &mut SELECTED_RANGE };

    ui.horizontal(|ui| {
        ui.heading("数据统计");
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            ui.horizontal(|ui| {
                ui.vertical(|ui| {
                    ui.heading(format!("{:.0}", focus_score));
                    ui.label("专注度得分");
                });

                ui.separator();

                ui.vertical(|ui| {
                    render_focus_insights(ui, stats, focus_score);
                });
            });
        });

    // 建议与洞察
    ui.add_space(styles::SPACING_MEDIUM);
    ui.heading("改进建议");
    render_recommendations(ui, stats);
}

fn render_focus_insights(ui: &mut egui::Ui, stats: &Statistics, focus_score: f32) {
    let mut insights = Vec::new();

    // 分析工作时长
    let avg_daily_work = stats.total_work_time.as_secs_f32() / stats.days_count as f32 / 3600.0;
    if avg_daily_work >= 6.0 {
        insights.push(("工作时长", "✓ 保持良好的工作时间", styles::COLOR_SUCCESS));
    } else if avg_daily_work < 4.0 {
        insights.push(("工作时长", "! 工作时间偏短", styles::COLOR_WARNING));
    }

    // 分析中断情况
    let interruption_rate = stats.interrupted_pomodoros as f32 / stats.total_pomodoros as f32;
    if interruption_rate < 0.1 {
        insights.push(("专注度", "✓ 很少被打断", styles::COLOR_SUCCESS));
    } else if interruption_rate > 0.3 {
        insights.push(("专注度", "! 经常被打断", styles::COLOR_ERROR));
    }

    // 分析工作规律
    if stats.regular_schedule {
        insights.push(("工作规律", "✓ 工作时间规律", styles::COLOR_SUCCESS));
    } else {
        insights.push(("工作规律", "! 工作时间不规律", styles::COLOR_WARNING));
    }

    // 显示洞察
    for (category, text, color) in insights {
        ui.horizontal(|ui| {
            ui.label(category);
            ui.colored_label(color, text);
        });
    }
}

fn render_recommendations(ui: &mut egui::Ui, stats: &Statistics) {
    let recommendations = generate_recommendations(stats);
    
    egui::ScrollArea::vertical()
        .max_height(200.0)
        .show(ui, |ui| {
            for recommendation in recommendations {
                Card::new()
                    .show(ui, |ui| {
                        ui.horizontal(|ui| {
                            ui.label("💡");
                            ui.colored_label(
                                styles::COLOR_INFO,
                                &recommendation,
                            );
                        });
                    });
            }
        });
}

fn generate_recommendations(stats: &Statistics) -> Vec<String> {
    let mut recommendations = Vec::new();

    // 基于工作时间的建议
    let avg_daily_work = stats.total_work_time.as_secs_f32() / stats.days_count as f32 / 3600.0;
    if avg_daily_work < 4.0 {
        recommendations.push(
            "建议增加每日工作时长，合理安排工作时间，保持规律的工作节奏。".to_string()
        );
    } else if avg_daily_work > 10.0 {
        recommendations.push(
            "您的工作时间偏长，请注意合理休息，避免过度疲劳。".to_string()
        );
    }

    // 基于中断的建议
    let interruption_rate = stats.interrupted_pomodoros as f32 / stats.total_pomodoros as f32;
    if interruption_rate > 0.3 {
        recommendations.push(
            "工作经常被打断，建议：\n1. 使用勿扰模式\n2. 找到安静的工作环境\n3. 与同事沟通，集中处理非紧急事务".to_string()
        );
    }

    // 基于最佳工作时段的建议
    if let Some((best_hour, _)) = stats.best_productive_hour {
        recommendations.push(
            format!("您在{}点工作效率最高，建议在此时间段安排重要任务。", best_hour)
        );
    }

    // 基于工作规律的建议
    if !stats.regular_schedule {
        recommendations.push(
            "工作时间不够规律，建议：\n1. 制定固定的工作计划\n2. 养成规律的作息习惯\n3. 使用番茄工作法提高时间利用效率".to_string()
        );
    }

    recommendations
}

#[derive(PartialEq, Copy, Clone)]
enum TimeRange {
    Week,
    Month,
    Quarter,
    Custom(chrono::DateTime<Local>, chrono::DateTime<Local>),
}

fn get_time_range(range: TimeRange) -> (chrono::DateTime<Local>, chrono::DateTime<Local>) {
    let now = Local::now();
    match range {
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
        TimeRange::Quarter => {
            let month = now.month();
            let quarter_start_month = ((month - 1) / 3) * 3 + 1;
            let start = now.date_naive()
                .with_month(quarter_start_month).unwrap()
                .with_day(1).unwrap()
                .and_hms_opt(0, 0, 0).unwrap()
                .and_local_timezone(Local).unwrap();
            (start, now)
        }
        TimeRange::Custom(start, end) => (start, end),
    }
}

fn get_category_color(category: &str) -> egui::Color32 {
    match category {
        "Development" => styles::COLOR_PRIMARY,
        "Design" => styles::COLOR_SECONDARY,
        "Meeting" => styles::COLOR_INFO,
        "Writing" => styles::COLOR_SUCCESS,
        "Research" => styles::COLOR_WARNING,
        _ => styles::COLOR_TEXT_SECONDARY,
    }
}

fn calculate_focus_score(stats: &Statistics) -> f32 {
    let mut score = 0.0;
    let mut weights = 0.0;

    // 计算完成率权重
    let completion_rate = (stats.total_pomodoros - stats.interrupted_pomodoros) as f32
        / stats.total_pomodoros as f32;
    score += completion_rate * 40.0;  // 40分权重
    weights += 40.0;

    // 计算工作规律性权重
    if stats.regular_schedule {
        score += 20.0;
    }
    weights += 20.0;

    // 计算平均专注度权重
    score += stats.average_focus_score * 40.0;  // 40分权重
    weights += 40.0;

    (score / weights) * 100.0
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

// 统计数据结构
#[derive(Default)]
struct Statistics {
    total_work_time: std::time::Duration,
    total_pomodoros: u32,
    interrupted_pomodoros: u32,
    average_focus_score: f32,
    work_time_change: f32,
    productivity_change: f32,
    days_count: u32,
    daily_productivity: Vec<(f64, f64)>,
    best_productive_hour: Option<(u32, f32)>,
    worst_productive_hour: Option<(u32, f32)>,
    project_stats: Vec<ProjectStatistics>,
    category_stats: HashMap<String, f64>,
    tag_stats: Vec<TagStatistics>,
    regular_schedule: bool,
}

struct ProjectStatistics {
    name: String,
    total_time: std::time::Duration,
    pomodoro_count: u32,
    target_pomodoros: Option<u32>,
    color: egui::Color32,
}

struct TagStatistics {
    name: String,
    count: u32,
}

#[cfg(test)]
mod tests {
    use super::*;
    use eframe::egui::Context;
    use std::time::Duration;

    #[test]
    fn test_statistics_rendering() {
        let ctx = Context::default();
        let mut app = TimeTrackerApp::test_new();

        ctx.run(|ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                render(&mut app, ui);
            });
        });
    }

    #[test]
    fn test_time_range_calculation() {
        let (start, end) = get_time_range(TimeRange::Week);
        assert_eq!(start.weekday(), chrono::Weekday::Mon);
        
        let (start, end) = get_time_range(TimeRange::Month);
        assert_eq!(start.day(), 1);
    }

    #[test]
    fn test_focus_score_calculation() {
        let mut stats = Statistics::default();
        stats.total_pomodoros = 10;
        stats.interrupted_pomodoros = 2;
        stats.regular_schedule = true;
        stats.average_focus_score = 0.8;

        let score = calculate_focus_score(&stats);
        assert!(score > 0.0 && score <= 100.0);
    }
}

                ui.selectable_value(selected_range, TimeRange::Week, "本周");
                ui.selectable_value(selected_range, TimeRange::Month, "本月");
                ui.selectable_value(selected_range, TimeRange::Quarter, "本季度");
            });
        });
    });

    let (start_time, end_time) = get_time_range(*selected_range);

    if let Ok(stats) = app.storage.get_statistics(start_time, end_time) {
        ui.horizontal(|ui| {
            // 左侧概览
            ui.vertical(|ui| {
                render_overview_stats(ui, &stats);
                ui.add_space(styles::SPACING_MEDIUM);
                render_productivity_chart(ui, &stats);
            }).min_width(250.0);

            ui.separator();

            // 右侧详细统计
            ui.vertical(|ui| {
                render_detailed_stats(ui, &stats);
            });
        });
    }
}

fn render_overview_stats(ui: &mut egui::Ui, stats: &Statistics) {
    ui.heading("概览");

    ui.horizontal(|ui| {
        Card::new()
            .show(ui, |ui| {
                ui.label("总工作时长");
                ui.heading(format_duration(stats.total_work_time));
            });

        Card::new()
            .show(ui, |ui| {
                ui.label("完成番茄数");
                ui.heading(format!("{}", stats.total_pomodoros));
            });

        Card::new()
            .show(ui, |ui| {
                ui.label("平均专注度");
                ui.heading(format!("{:.1}%", stats.average_focus_score * 100.0));
            });
    });

    // 显示趋势变化
    ui.add_space(styles::SPACING_SMALL);
    ui.horizontal(|ui| {
        if stats.work_time_change > 0.0 {
            ui.colored_label(
                styles::COLOR_SUCCESS,
                format!("↑ 工作时长较上期增长 {:.1}%", stats.work_time_change * 100.0),
            );
        } else if stats.work_time_change < 0.0 {
            ui.colored_label(
                styles::COLOR_WARNING,
                format!("↓ 工作时长较上期减少 {:.1}%", -stats.work_time_change * 100.0),
            );
        }

        if stats.productivity_change > 0.05 {
            ui.colored_label(
                styles::COLOR_SUCCESS,
                format!("↑ 专注度提升 {:.1}%", stats.productivity_change * 100.0),
            );
        } else if stats.productivity_change < -0.05 {
            ui.colored_label(
                styles::COLOR_WARNING,
                format!("↓ 专注度下降 {:.1}%", -stats.productivity_change * 100.0),
            );
        }
    });
}

fn render_productivity_chart(ui: &mut egui::Ui, stats: &Statistics) {
    ui.heading("生产力趋势");

    // 显示每天的生产力得分趋势
    Chart::new(stats.daily_productivity.clone())
        .with_size(ui.available_width(), 200.0)
        .with_color(styles::COLOR_PRIMARY)
        .show(ui);

    // 显示最佳和最差时段
    ui.horizontal(|ui| {
        if let Some((best_hour, score)) = stats.best_productive_hour {
            Card::new()
                .show(ui, |ui| {
                    ui.colored_label(styles::COLOR_SUCCESS, "最佳工作时段");
                    ui.heading(format!("{}:00", best_hour));
                    ui.label(format!("平均专注度 {:.1}%", score * 100.0));
                });
        }

        if let Some((worst_hour, score)) = stats.worst_productive_hour {
            Card::new()
                .show(ui, |ui| {
                    ui.colored_label(styles::COLOR_ERROR, "易分心时段");
                    ui.heading(format!("{}:00", worst_hour));
                    ui.label(format!("平均专注度 {:.1}%", score * 100.0));
                });
        }
    });
}

fn render_detailed_stats(ui: &mut egui::Ui, stats: &Statistics) {
    // 项目统计
    ui.collapsing("项目统计", |ui| {
        for project_stat in &stats.project_stats {
            Card::new()
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        ui.label(&project_stat.name);
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            ui.label(format_duration(project_stat.total_time));
                            ui.label(format!("{}次番茄", project_stat.pomodoro_count));
                        });
                    });

                    // 显示完成进度
                    if let Some(target) = project_stat.target_pomodoros {
                        let progress = project_stat.pomodoro_count as f32 / target as f32;
                        ProgressBar::new(progress)
                            .show_percentage(true)
                            .with_color(project_stat.color)
                            .show(ui);
                    }
                });
        }
    });

    // 应用分类统计
    ui.collapsing("应用分类统计", |ui| {
        let total_time: f64 = stats.category_stats.values().sum();
        
        for (category, &time) in &stats.category_stats {
            let percentage = time / total_time;
            ui.horizontal(|ui| {
                ui.label(category);
                ProgressBar::new(percentage as f32)
                    .show_percentage(true)
                    .with_color(get_category_color(category))
                    .show(ui);
            });
        }
    });

    // 标签统计
    ui.collapsing("标签统计", |ui| {
        for tag_stat in &stats.tag_stats {
            ui.horizontal(|ui| {
                Tag::new(&tag_stat.name)
                    .with_color(styles::COLOR_PRIMARY)
                    .show(ui);
                ui.label(format!("{}次使用", tag_stat.count));
            });
        }
    });

    // 专注度分析
    ui.add_space(styles::SPACING_MEDIUM);
    ui.heading("专注度分析");

    let focus_score = calculate_focus_score(stats);
    Card::new()
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.vertical(|ui| {
                    ui.heading(format!("{:.0}", focus_score));
                    ui.label("专注度得分");
                });

                ui.separator();

                ui.vertical(|ui| {
                    render_focus_insights(ui, stats, focus_score);
                });
            });
        });