//src/ui/views/statistics.rs

use eframe::egui;
use crate::ui::{TimeTrackerApp, styles};
use super::components::{Card, Chart};
use crate::ui::components::dialog::DateRangeDialog;
use chrono::NaiveDate;

#[derive(PartialEq)]
enum TimeRange {
    LastWeek,
    LastMonth,
    LastThreeMonths,
    LastYear,
    Custom(Option<(NaiveDate, NaiveDate)>),
}

impl TimeRange {
    fn as_str(&self) -> String {
        match self {
            TimeRange::LastWeek => "最近一周".to_string(),
            TimeRange::LastMonth => "最近一个月".to_string(),
            TimeRange::LastThreeMonths => "最近三个月".to_string(),
            TimeRange::LastYear => "最近一年".to_string(),
            TimeRange::Custom(Some((start, end))) => {
                format!("{}至{}", start.format("%Y-%m-%d"), end.format("%Y-%m-%d"))
            }
            TimeRange::Custom(None) => "自定义范围".to_string(),
        }
    }
}

pub fn render(app: &mut TimeTrackerApp, ctx: &egui::Context, ui: &mut egui::Ui) {
    static mut SELECTED_RANGE: Option<TimeRange> = None;
    static mut DATE_RANGE_DIALOG: Option<DateRangeDialog> = None;
    
    let selected_range = unsafe {
        if SELECTED_RANGE.is_none() {
            SELECTED_RANGE = Some(TimeRange::LastWeek);
        }
        SELECTED_RANGE.as_mut().unwrap()
    };

    let date_range_dialog = unsafe {
        if DATE_RANGE_DIALOG.is_none() {
            DATE_RANGE_DIALOG = Some(DateRangeDialog::default());
        }
        DATE_RANGE_DIALOG.as_mut().unwrap()
    };

    ui.heading("统计分析");
    ui.separator();

    ui.horizontal(|ui| {
        ui.label("时间范围:");
        egui::ComboBox::from_id_source("time_range_selector")
            .selected_text(selected_range.as_str())
            .show_ui(ui, |ui| {
                ui.selectable_value(selected_range, TimeRange::LastWeek, "最近一周");
                ui.selectable_value(selected_range, TimeRange::LastMonth, "最近一个月");
                ui.selectable_value(selected_range, TimeRange::LastThreeMonths, "最近三个月");
                ui.selectable_value(selected_range, TimeRange::LastYear, "最近一年");
                ui.selectable_value(selected_range, TimeRange::Custom(None), "自定义范围");
            });
            
        if matches!(selected_range, TimeRange::Custom(_)) {
            if ui.button("选择日期范围").clicked() {
                date_range_dialog.open = true;
                date_range_dialog.on_close = Some(Box::new(|result| {
                    if let Some((start, end)) = result {
                        unsafe {
                            *SELECTED_RANGE = Some(TimeRange::Custom(Some((start, end))));
                        }
                    }
                }));
            }
        }
    });

    // 显示日期选择对话框
    date_range_dialog.show(ctx);

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
        .show(ui, |ui| {
            // 添加表格标题
            ui.horizontal(|ui| {
                ui.style_mut().spacing.item_spacing.x = 40.0;
                
                ui.label("日期");
                ui.label("完成番茄数");
                ui.label("专注时长");
                ui.label("休息时长");
                ui.label("效率得分");
            });
            
            ui.separator();
            
            // 显示每天的统计数据
            for i in 0..7 {
                ui.horizontal(|ui| {
                    ui.style_mut().spacing.item_spacing.x = 40.0;
                    
                    // 这里暂时使用模拟数据，之后需要从数据库获取实际数据
                    ui.label(format!("2024-03-{:02}", i + 1));
                    ui.label(format!("{}", 8 - i));
                    ui.label(format!("{}小时", 4 - i/2));
                    ui.label(format!("{}分钟", 30 + i * 5));
                    ui.label(format!("{}%", 95 - i * 3));
                });
                
                if i < 6 {
                    ui.add_space(4.0);
                    ui.separator();
                    ui.add_space(4.0);
                }
            }
        });
}