use super::base::{Dialog, DialogContext};
use crate::error::Result;
use chrono::{DateTime, Local};
use eframe::egui;
use crate::ui::components::Button;
use crate::ui::styles;
use rfd::FileDialog;

#[derive(Debug, Clone, PartialEq)]
pub enum ExportFormat {
    CSV,
    JSON,
    Excel,
}

pub struct DateRange {
    pub start: DateTime<Local>,
    pub end: DateTime<Local>,
}

impl Default for DateRange {
    fn default() -> Self {
        let now = Local::now();
        Self {
            start: now - chrono::Duration::days(30),
            end: now,
        }
    }
}

pub struct ExportDialog {
    pub format: ExportFormat,
    pub path: String,
    pub date_range: DateRange,
    pub include_app_usage: bool,
    pub include_pomodoros: bool,
    pub include_statistics: bool,
}

impl Default for ExportDialog {
    fn default() -> Self {
        Self {
            format: ExportFormat::CSV,
            path: String::new(),
            date_range: DateRange::default(),
            include_app_usage: true,
            include_pomodoros: true,
            include_statistics: true,
        }
    }
}

impl Dialog for ExportDialog {
    fn show(&mut self, ctx: &egui::Context, dialog_ctx: &mut DialogContext) -> bool {
        let mut is_open = true;

        egui::Window::new("导出数据")
            .collapsible(false)
            .resizable(false)
            .open(&mut is_open)
            .show(ctx, |ui| {
                ui.spacing_mut().item_spacing = egui::vec2(styles::SPACING_MEDIUM, styles::SPACING_MEDIUM);

                // 导出格式
                ui.horizontal(|ui| {
                    ui.label("格式");
                    ui.radio_value(&mut self.format, ExportFormat::CSV, "CSV");
                    ui.radio_value(&mut self.format, ExportFormat::JSON, "JSON");
                    ui.radio_value(&mut self.format, ExportFormat::Excel, "Excel");
                });

                // 导出路径
                ui.horizontal(|ui| {
                    ui.label("导出路径");
                    ui.text_edit_singleline(&mut self.path);
                    if Button::new("选择")
                        .with_style(styles::ButtonStyle::outlined())
                        .show(ui)
                        .clicked()
                    {
                        if let Some(path) = FileDialog::new()
                            .add_filter("导出文件", &["csv", "json", "xlsx"])
                            .save_file()
                        {
                            self.path = path.display().to_string();
                        }
                    }
                });

                // 日期范围
                ui.label("日期范围");
                ui.horizontal(|ui| {
                    ui.label(format!("从 {} 到 {}",
                        self.date_range.start.format("%Y-%m-%d"),
                        self.date_range.end.format("%Y-%m-%d")
                    ));
                });

                // 导出内容选择
                ui.label("导出内容");
                ui.checkbox(&mut self.include_app_usage, "应用使用记录");
                ui.checkbox(&mut self.include_pomodoros, "番茄钟记录");
                ui.checkbox(&mut self.include_statistics, "统计数据");

                ui.add_space(ui.spacing().item_spacing.y);

                // 按钮区域
                ui.horizontal(|ui| {
                    if Button::new("取消")
                        .with_style(styles::ButtonStyle::outlined())
                        .show(ui)
                        .clicked()
                    {
                        dialog_ctx.pop_dialog();
                    }

                    if Button::new("导出")
                        .enabled(!self.path.is_empty())
                        .show(ui)
                        .clicked()
                    {
                        self.export_data(dialog_ctx);
                        dialog_ctx.pop_dialog();
                    }
                });
            });

        is_open
    }
}

impl ExportDialog {
    fn export_data(&self, dialog_ctx: &mut DialogContext) {
        let result = match self.format {
            ExportFormat::CSV => self.export_csv(dialog_ctx),
            ExportFormat::JSON => self.export_json(dialog_ctx),
            ExportFormat::Excel => self.export_excel(dialog_ctx),
        };

        if let Err(e) = result {
            dialog_ctx.show_error(format!("导出失败: {}", e));
        }
    }

    fn export_csv(&self, dialog_ctx: &mut DialogContext) -> Result<()> {
        // 实现 CSV 导出逻辑
        Ok(())
    }

    fn export_json(&self, dialog_ctx: &mut DialogContext) -> Result<()> {
        // 实现 JSON 导出逻辑
        Ok(())
    }

    fn export_excel(&self, dialog_ctx: &mut DialogContext) -> Result<()> {
        // 实现 Excel 导出逻辑
        Ok(())
    }
} 