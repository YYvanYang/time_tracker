use super::Button;
use crate::error::Result;
use crate::ui::{self, styles, TimeTrackerApp, DialogHandler, DialogContext};
use crate::pomodoro::PomodoroConfig;
use eframe::egui;
use crate::error::TimeTrackerError;
use rfd::FileDialog;
use chrono::{NaiveDate, Local, Datelike, Timelike, DateTime};
use open;
use std::sync::{Arc, Mutex};

// 基础对话框特征
pub trait Dialog {
    fn show(&mut self, ctx: &egui::Context, dialog_ctx: &mut ui::DialogContext) -> bool;
    fn validate(&self) -> Result<()> {
        Ok(())  // 默认实现
    }
}

// 项目对话框
pub struct ProjectDialog {
    pub title: String,
    pub name: String,
    pub description: String,
    pub color: egui::Color32,
    pub on_save: Option<Box<dyn FnOnce(&mut TimeTrackerApp, String, String, egui::Color32) -> Result<()> + Send>>,
}

impl ProjectDialog {
    pub fn new(title: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            name: String::new(),
            description: String::new(),
            color: styles::COLOR_PRIMARY,
            on_save: None,
        }
    }

    pub fn show_custom<F>(&mut self, ctx: &egui::Context, content: F) -> bool 
    where
        F: FnOnce(&mut egui::Ui)
    {
        let mut is_open = true;
        egui::Window::new(&self.title)
            .collapsible(false)
            .resizable(false)
            .open(&mut is_open)
            .show(ctx, |ui| {
                content(ui);
            });
        is_open
    }
}

impl Dialog for ProjectDialog {
    fn show(&mut self, ctx: &egui::Context, dialog_ctx: &mut ui::DialogContext) -> bool {
        egui::Window::new(&self.title)
            .collapsible(false)
            .resizable(false)
            .default_width(400.0)
            .show(ctx, |ui| {
                ui.spacing_mut().item_spacing = egui::vec2(styles::SPACING_MEDIUM, styles::SPACING_MEDIUM);

                // 项目名称
                ui.label("名称");
                ui.text_edit_singleline(&mut self.name);

                // 项目描述
                ui.label("描述");
                ui.text_edit_multiline(&mut self.description);

                // 项目颜色
                ui.horizontal(|ui| {
                    ui.label("颜色");
                    egui::color_picker::color_edit_button_srgba(
                        ui,
                        &mut self.color,
                        egui::color_picker::Alpha::Opaque,
                    );
                });

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

                    if Button::new("保存")
                        .enabled(!self.name.is_empty())
                        .show(ui)
                        .clicked()
                    {
                        if let Some(on_save) = self.on_save.take() {
                            if let Err(e) = on_save(
                                dialog_ctx.app,
                                self.name.clone(),
                                self.description.clone(),
                                self.color,
                            ) {
                                dialog_ctx.show_error(e.to_string());
                            }
                        }
                        dialog_ctx.pop_dialog();
                    }
                });
            });
        true
    }
}

impl std::fmt::Debug for ProjectDialog {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ProjectDialog")
            .field("title", &self.title)
            .field("name", &self.name)
            .field("description", &self.description)
            .field("color", &self.color)
            .field("on_save", &format_args!("<callback>"))
            .finish()
    }
}

// 标签对话框
pub struct TagDialog {
    pub name: String,
    pub color: egui::Color32,
    pub on_save: Option<Box<dyn FnOnce(&mut TimeTrackerApp, String, egui::Color32) -> Result<()> + Send>>,
}

impl TagDialog {
    pub fn new() -> Self {
        Self {
            name: String::new(),
            color: styles::COLOR_PRIMARY,
            on_save: None,
        }
    }
}

impl std::fmt::Debug for TagDialog {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TagDialog")
            .field("name", &self.name)
            .field("color", &self.color)
            .field("on_save", &format_args!("<callback>"))
            .finish()
    }
}

impl Dialog for TagDialog {
    fn show(&mut self, ctx: &egui::Context, dialog_ctx: &mut ui::DialogContext) -> bool {
        let mut is_open = true;
        egui::Window::new("添加标签")
            .open(&mut is_open)
            .show(ctx, |ui| {
                ui.spacing_mut().item_spacing = egui::vec2(styles::SPACING_MEDIUM, styles::SPACING_MEDIUM);

                // 标签名称
                ui.label("名称");
                ui.text_edit_singleline(&mut self.name);

                // 标签颜色
                ui.horizontal(|ui| {
                    ui.label("颜色");
                    egui::color_picker::color_edit_button_srgba(
                        ui,
                        &mut self.color,
                        egui::color_picker::Alpha::Opaque,
                    );
                });

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

                    if Button::new("保存")
                        .enabled(!self.name.is_empty())
                        .show(ui)
                        .clicked()
                    {
                        if let Some(on_save) = self.on_save.take() {
                            if let Err(e) = on_save(dialog_ctx.app, self.name.clone(), self.color) {
                                dialog_ctx.show_error(e.to_string());
                            }
                        }
                        dialog_ctx.pop_dialog();
                    }
                });
            });
        is_open
    }
}

// 导出对话框
pub struct ExportDialog {
    pub format: ExportFormat,
    pub path: String,
    pub date_range: DateRange,
    pub include_app_usage: bool,
    pub include_pomodoros: bool,
    pub include_statistics: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ExportFormat {
    CSV,
    JSON,
    Excel,
}

#[derive(Debug)]
pub struct DateRange {
    pub start: chrono::DateTime<chrono::Local>,
    pub end: chrono::DateTime<chrono::Local>,
}

impl Default for ExportDialog {
    fn default() -> Self {
        let now = chrono::Local::now();
        let start = now - chrono::Duration::days(30);
        Self {
            format: ExportFormat::CSV,
            path: String::new(),
            date_range: DateRange { start, end: now },
            include_app_usage: true,
            include_pomodoros: true,
            include_statistics: true,
        }
    }
}

impl Dialog for ExportDialog {
    fn show(&mut self, ctx: &egui::Context, dialog_ctx: &mut ui::DialogContext) -> bool {
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
                            .add_filter("JSON", &["json"])
                            .save_file()
                        {
                            self.path = path.display().to_string();
                        }
                    }
                });

                // 日期范围
                ui.label("日期范围");
                ui.horizontal(|ui| {
                    // 显示当前选择的日期范围
                    ui.label(format!("从 {} 到 {}",
                        self.date_range.start.format("%Y-%m-%d"),
                        self.date_range.end.format("%Y-%m-%d")
                    ));
                    
                    if ui.button("选择日期范围").clicked() {
                        let start = self.date_range.start;
                        let end = self.date_range.end;
                        let date_picker = DateRangeDialog::new()
                            .with_dates(
                                start.date_naive(),
                                end.date_naive()
                            );

                        dialog_ctx.app.push_dialog(Box::new(date_picker));
                    }
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
                        let result = match self.format {
                            ExportFormat::CSV => self.export_csv(dialog_ctx.app),
                            ExportFormat::JSON => self.export_json(dialog_ctx.app),
                            ExportFormat::Excel => self.export_excel(dialog_ctx.app),
                        };

                        if let Err(e) = result {
                            dialog_ctx.show_error(format!("导出失败: {}", e));
                        }
                        dialog_ctx.pop_dialog();
                    }
                });
            });
        is_open
    }
}

impl ExportDialog {
    fn export_data(&self, app: &mut TimeTrackerApp) {
        let result = match self.format {
            ExportFormat::CSV => self.export_csv(app),
            ExportFormat::JSON => self.export_json(app),
            ExportFormat::Excel => self.export_excel(app),
        };

        if let Err(e) = result {
            app.show_error(format!("导出失败: {}", e));
        }
    }

    fn export_csv(&self, app: &mut TimeTrackerApp) -> Result<()> {
        use std::fs::File;
        use csv::Writer;

        let file = File::create(&self.path)?;
        let mut writer = Writer::from_writer(file);

        // 导出应用使用记录
        if self.include_app_usage {
            writer.write_record(&["时间", "应用名称", "窗口标题", "使用时长(秒)"])?;
            
            let records = app.storage.lock().unwrap()
                .get_app_usage_records(self.date_range.start, self.date_range.end)?;
            
            for record in records {
                writer.write_record(&[
                    record.start_time.format("%Y-%m-%d %H:%M:%S").to_string(),
                    record.app_name,
                    record.window_title,
                    record.duration.as_secs().to_string(),
                ])?;
            }
            
            // 写入空行
            writer.write_record::<_, String>(&[])?;
        }

        // 导出番茄钟记录
        if self.include_pomodoros {
            writer.write_record(&["开始时间", "结束时间", "状态", "备注"])?;
            
            let records = app.storage.lock().unwrap()
                .get_pomodoro_records(self.date_range.start, self.date_range.end)?;
            
            for record in records {
                writer.write_record(&[
                    record.start_time.format("%Y-%m-%d %H:%M:%S").to_string(),
                    record.end_time.format("%Y-%m-%d %H:%M:%S").to_string(),
                    record.status.to_string(),
                    record.notes.as_deref().unwrap_or("").to_string(),
                ])?;
            }
        }

        writer.flush()?;
        Ok(())
    }

    fn export_json(&self, app: &mut TimeTrackerApp) -> Result<()> {
        use std::fs::File;
        use std::io::Write;
        use serde_json::{json, Value};

        let mut export_data = json!({
            "export_time": chrono::Local::now().to_rfc3339(),
            "date_range": {
                "start": self.date_range.start.to_rfc3339(),
                "end": self.date_range.end.to_rfc3339(),
            }
        });

        let storage = app.storage.lock().unwrap();

        if self.include_app_usage {
            let records = storage.get_app_usage_records(
                self.date_range.start, 
                self.date_range.end
            )?;
            
            export_data["app_usage"] = json!(records);
        }

        if self.include_pomodoros {
            let records = storage.get_pomodoro_records(
                self.date_range.start, 
                self.date_range.end
            )?;
            
            export_data["pomodoros"] = json!(records);
        }

        if self.include_statistics {
            let stats = storage.get_statistics(
                self.date_range.start, 
                self.date_range.end
            )?;
            
            export_data["statistics"] = json!(stats);
        }

        let file = File::create(&self.path)?;
        serde_json::to_writer_pretty(file, &export_data)?;
        
        Ok(())
    }

    fn export_excel(&self, app: &mut TimeTrackerApp) -> Result<()> {
        use xlsxwriter::Workbook;
        use xlsxwriter::worksheet::DateTime;
        use chrono::Timelike;

        let workbook = Workbook::new(&self.path)?;
        let storage = app.storage.lock().unwrap();

        // 辅助函数：将 chrono::DateTime 转换为 xlsxwriter::DateTime
        fn to_excel_datetime(dt: chrono::DateTime<Local>) -> DateTime {
            DateTime::new(
                dt.year() as i16,
                dt.month() as i8,
                dt.day() as i8,
                dt.hour() as i8,
                dt.minute() as i8,
                dt.second() as f64,
            )
        }

        // 应用使用记录工作表
        if self.include_app_usage {
            let mut sheet = workbook.add_worksheet(Some("应用使用记录"))?;
            
            // 写入表头
            sheet.write_string(0, 0, "时间", None)?;
            sheet.write_string(0, 1, "应用名称", None)?;
            sheet.write_string(0, 2, "窗口标题", None)?;
            sheet.write_string(0, 3, "使用时长(秒)", None)?;

            let records = storage.get_app_usage_records(
                self.date_range.start, 
                self.date_range.end
            )?;

            for (i, record) in records.iter().enumerate() {
                let row = (i + 1) as u32;
                sheet.write_datetime(row, 0, &to_excel_datetime(record.start_time), None)?;
                sheet.write_string(row, 1, &record.app_name, None)?;
                sheet.write_string(row, 2, &record.window_title, None)?;
                sheet.write_number(row, 3, record.duration.as_secs() as f64, None)?;
            }
        }

        // 番茄钟记录工作表
        if self.include_pomodoros {
            let mut sheet = workbook.add_worksheet(Some("番茄钟记录"))?;
            
            // 写入表头
            sheet.write_string(0, 0, "开始时间", None)?;
            sheet.write_string(0, 1, "结束时间", None)?;
            sheet.write_string(0, 2, "状态", None)?;
            sheet.write_string(0, 3, "备注", None)?;

            let records = storage.get_pomodoro_records(
                self.date_range.start, 
                self.date_range.end
            )?;

            for (i, record) in records.iter().enumerate() {
                let row = (i + 1) as u32;
                sheet.write_datetime(row, 0, &to_excel_datetime(record.start_time), None)?;
                sheet.write_datetime(row, 1, &to_excel_datetime(record.end_time), None)?;
                sheet.write_string(row, 2, &record.status.to_string(), None)?;
                if let Some(notes) = &record.notes {
                    sheet.write_string(row, 3, notes, None)?;
                }
            }
        }

        // 统计数据工作表
        if self.include_statistics {
            let mut sheet = workbook.add_worksheet(Some("统计数据"))?;
            
            let stats = storage.get_statistics(
                self.date_range.start, 
                self.date_range.end
            )?;

            // 写入统计数据...
            sheet.write_string(0, 0, "统计项", None)?;
            sheet.write_string(0, 1, "数值", None)?;
            
            let mut row = 1;
            for (key, value) in stats {
                sheet.write_string(row, 0, &key, None)?;
                sheet.write_string(row, 1, &value.to_string(), None)?;
                row += 1;
            }
        }

        workbook.close()?;
        Ok(())
    }
}

impl std::fmt::Debug for ExportDialog {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ExportDialog")
            .field("format", &self.format)
            .field("path", &self.path)
            .field("date_range", &self.date_range)
            .field("include_app_usage", &self.include_app_usage)
            .field("include_pomodoros", &self.include_pomodoros)
            .field("include_statistics", &self.include_statistics)
            .finish()
    }
}

// 设置对话框
pub struct SettingsDialog {
    pub general_settings: GeneralSettings,
    pub pomodoro_settings: PomodoroSettings,
    pub notification_settings: NotificationSettings,
    pub backup_settings: BackupSettings,
}

#[derive(Debug)]
pub struct GeneralSettings {
    pub autostart: bool,
    pub minimize_to_tray: bool,
    pub check_updates: bool,
    pub language: String,
}

#[derive(Debug)]
pub struct PomodoroSettings {
    pub work_duration: std::time::Duration,
    pub short_break_duration: std::time::Duration,
    pub long_break_duration: std::time::Duration,
    pub long_break_interval: u32,
    pub auto_start_breaks: bool,
    pub auto_start_pomodoros: bool,
}

#[derive(Debug)]
pub struct NotificationSettings {
    pub enabled: bool,
    pub sound_enabled: bool,
    pub sound_volume: u8,
}

#[derive(Debug)]
pub struct BackupSettings {
    pub enabled: bool,
    pub interval: u32,
    pub max_backup_count: u32,
    pub backup_path: String,
}

impl SettingsDialog {
    pub fn new(config: &crate::config::Config) -> Self {
        Self {
            general_settings: GeneralSettings {
                autostart: config.general.autostart,
                minimize_to_tray: config.general.minimize_to_tray,
                check_updates: config.general.check_updates,
                language: config.general.language.clone(),
            },
            pomodoro_settings: PomodoroSettings {
                work_duration: config.pomodoro.work_duration,
                short_break_duration: config.pomodoro.short_break_duration,
                long_break_duration: config.pomodoro.long_break_duration,
                long_break_interval: config.pomodoro.long_break_interval,
                auto_start_breaks: config.pomodoro.auto_start_breaks,
                auto_start_pomodoros: config.pomodoro.auto_start_pomodoros,
            },
            notification_settings: NotificationSettings {
                enabled: true,
                sound_enabled: config.pomodoro.sound_enabled,
                sound_volume: config.pomodoro.sound_volume,
            },
            backup_settings: BackupSettings {
                enabled: config.storage.backup_enabled,
                interval: config.storage.backup_interval.as_secs() as u32 / 3600,
                max_backup_count: config.storage.max_backup_count,
                backup_path: config.storage.data_dir.to_string_lossy().into_owned(),
            },
        }
    }
}

impl Dialog for SettingsDialog {
    fn show(&mut self, ctx: &egui::Context, dialog_ctx: &mut ui::DialogContext) -> bool {
        let mut is_open = true;
        let mut should_close = false;
        egui::Window::new("设置")
            .collapsible(false)
            .resizable(false)
            .open(&mut is_open)
            .show(ctx, |ui| {
                egui::ScrollArea::vertical().show(ui, |ui| {
                    self.show_general_settings(ui);
                    ui.add_space(styles::SPACING_LARGE);
                    self.show_pomodoro_settings(ui);
                    ui.add_space(styles::SPACING_LARGE);
                    self.show_notification_settings(ui);
                    ui.add_space(styles::SPACING_LARGE);
                    self.show_backup_settings(ui);
                    ui.add_space(styles::SPACING_LARGE);

                    // 按钮区域
                    ui.horizontal(|ui| {
                        if Button::new("取消")
                            .with_style(styles::ButtonStyle::outlined())
                            .show(ui)
                            .clicked()
                        {
                            dialog_ctx.pop_dialog();
                        }

                        if Button::new("保存")
                            .with_style(styles::ButtonStyle::primary())
                            .show(ui)
                            .clicked()
                        {
                            if let Err(e) = self.save_settings(dialog_ctx.app) {
                                dialog_ctx.show_error(format!("保存设置失败: {}", e));
                            } else {
                                dialog_ctx.pop_dialog();
                            }
                        }
                    });
                });
            });
        is_open
    }
}

impl SettingsDialog {
    fn show_general_settings(&mut self, ui: &mut egui::Ui) {
        ui.heading("常规设置");
        ui.checkbox(&mut self.general_settings.autostart, "开机自启动");
        ui.checkbox(&mut self.general_settings.minimize_to_tray, "最小化到托盘");
        ui.horizontal(|ui| {
            ui.label("语言");
            egui::ComboBox::from_id_source("language")
                .selected_text(&self.general_settings.language)
                .show_ui(ui, |ui| {
                    ui.selectable_value(&mut self.general_settings.language, "zh-CN".to_string(), "简体中文");
                    ui.selectable_value(&mut self.general_settings.language, "en".to_string(), "English");
                });
        });
    }

    fn show_pomodoro_settings(&mut self, ui: &mut egui::Ui) {
        ui.heading("番茄钟设置");
        
        let mut work_minutes = self.pomodoro_settings.work_duration.as_secs() / 60;
        ui.horizontal(|ui| {
            ui.label("工作时长(分钟)");
            if ui.add(egui::DragValue::new(&mut work_minutes)
                .clamp_range(1..=120))
                .changed() 
            {
                self.pomodoro_settings.work_duration = std::time::Duration::from_secs(work_minutes * 60);
            }
        });

        let mut short_break_minutes = self.pomodoro_settings.short_break_duration.as_secs() / 60;
        ui.horizontal(|ui| {
            ui.label("短休息时长(分钟)");
            if ui.add(egui::DragValue::new(&mut short_break_minutes)
                .clamp_range(1..=30))
                .changed()
            {
                self.pomodoro_settings.short_break_duration = std::time::Duration::from_secs(short_break_minutes * 60);
            }
        });

        let mut long_break_minutes = self.pomodoro_settings.long_break_duration.as_secs() / 60;
        ui.horizontal(|ui| {
            ui.label("长休息时长(分钟)");
            if ui.add(egui::DragValue::new(&mut long_break_minutes)
                .clamp_range(5..=60))
                .changed()
            {
                self.pomodoro_settings.long_break_duration = std::time::Duration::from_secs(long_break_minutes * 60);
            }
        });

        ui.horizontal(|ui| {
            ui.label("长休息间隔");
            ui.add(egui::DragValue::new(&mut self.pomodoro_settings.long_break_interval)
                .clamp_range(1..=10));
        });

        ui.checkbox(&mut self.pomodoro_settings.auto_start_breaks, "自动开始休息");
        ui.checkbox(&mut self.pomodoro_settings.auto_start_pomodoros, "休息后自动开始工作");
    }

    fn show_notification_settings(&mut self, ui: &mut egui::Ui) {
        ui.heading("通知设置");
        ui.checkbox(&mut self.notification_settings.enabled, "启用通知");
        ui.checkbox(&mut self.notification_settings.sound_enabled, "启用声音");
        
        if self.notification_settings.sound_enabled {
            ui.horizontal(|ui| {
                ui.label("音量");
                ui.add(egui::Slider::new(&mut self.notification_settings.sound_volume, 0..=100));
            });
        }
    }

    fn show_backup_settings(&mut self, ui: &mut egui::Ui) {
        ui.heading("备份设置");
        ui.checkbox(&mut self.backup_settings.enabled, "启用自动备份");
        
        if self.backup_settings.enabled {
            ui.horizontal(|ui| {
                ui.label("备份间隔(小时)");
                ui.add(egui::DragValue::new(&mut self.backup_settings.interval)
                    .clamp_range(1..=168));
            });

            ui.horizontal(|ui| {
                ui.label("保留数据天数");
                ui.add(egui::DragValue::new(&mut self.backup_settings.max_backup_count)
                    .clamp_range(1..=365));
            });

            ui.horizontal(|ui| {
                ui.label("备份路径");
                ui.text_edit_singleline(&mut self.backup_settings.backup_path);
                if Button::new("选择")
                    .with_style(styles::ButtonStyle::outlined())
                    .show(ui)
                    .clicked()
                {
                    if let Some(path) = FileDialog::new()
                        .set_title("选择备份路径")
                        .pick_folder()
                    {
                        self.backup_settings.backup_path = path.display().to_string();
                    }
                }
            });
        }
    }

    fn save_settings(&self, app: &mut TimeTrackerApp) -> Result<()> {
        // 获取配置的可变引用
        let mut config = app.config.lock().unwrap();
        
        // 保存常规设置
        config.general.autostart = self.general_settings.autostart;
        config.general.minimize_to_tray = self.general_settings.minimize_to_tray;
        config.general.check_updates = self.general_settings.check_updates;
        config.general.language = self.general_settings.language.clone();
        
        // 保存番茄钟设置
        config.pomodoro.work_duration = self.pomodoro_settings.work_duration;
        config.pomodoro.short_break_duration = self.pomodoro_settings.short_break_duration;
        config.pomodoro.long_break_duration = self.pomodoro_settings.long_break_duration;
        config.pomodoro.long_break_interval = self.pomodoro_settings.long_break_interval;
        config.pomodoro.auto_start_breaks = self.pomodoro_settings.auto_start_breaks;
        config.pomodoro.auto_start_pomodoros = self.pomodoro_settings.auto_start_pomodoros;
        config.pomodoro.sound_enabled = self.notification_settings.sound_enabled;
        config.pomodoro.sound_volume = self.notification_settings.sound_volume;
        
        // 保存备份设置
        config.storage.backup_enabled = self.backup_settings.enabled;
        config.storage.backup_interval = std::time::Duration::from_secs(
            self.backup_settings.interval as u64 * 3600
        );
        config.storage.max_backup_count = self.backup_settings.max_backup_count;
        config.storage.data_dir = std::path::PathBuf::from(&self.backup_settings.backup_path);
        
        // 保存配置到文件
        config.save()?;
        
        // 应用新的设置
        #[cfg(target_os = "windows")]
        if self.general_settings.autostart {
            crate::platform::windows::set_autostart(true)?;
        } else {
            crate::platform::windows::set_autostart(false)?;
        }

        #[cfg(not(target_os = "windows"))]
        if self.general_settings.autostart {
            log::warn!("Autostart is not supported on this platform");
        }
        
        // 重新配置番茄钟计时器
        {
            let mut pomodoro_timer = app.pomodoro_timer.lock().unwrap();
            pomodoro_timer.set_config(PomodoroConfig {
                work_duration: config.pomodoro.work_duration,
                short_break_duration: config.pomodoro.short_break_duration,
                long_break_duration: config.pomodoro.long_break_duration,
                long_break_interval: config.pomodoro.long_break_interval,
                auto_start_breaks: config.pomodoro.auto_start_breaks,
                auto_start_pomodoros: config.pomodoro.auto_start_pomodoros,
                sound_enabled: config.pomodoro.sound_enabled,
                sound_volume: config.pomodoro.sound_volume,
            });
        }
        
        // 重新配置存储
        let mut storage = app.storage.lock().unwrap();
        storage.update_config(config.storage.clone())?;
        
        Ok(())
    }
}

impl std::fmt::Debug for SettingsDialog {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SettingsDialog")
            .field("general_settings", &self.general_settings)
            .field("pomodoro_settings", &self.pomodoro_settings)
            .field("notification_settings", &self.notification_settings)
            .field("backup_settings", &self.backup_settings)
            .finish()
    }
}

pub struct ConfirmationDialog {
    pub title: String,
    pub message: String,
    pub on_confirm: Option<Box<dyn FnOnce(&mut TimeTrackerApp) -> Result<()> + Send>>,
    pub on_cancel: Option<Box<dyn FnOnce(&mut TimeTrackerApp) -> Result<()> + Send>>,
}

impl Dialog for ConfirmationDialog {
    fn show(&mut self, ctx: &egui::Context, dialog_ctx: &mut ui::DialogContext) -> bool {
        if let Err(e) = self.validate() {
            dialog_ctx.show_error(e.to_string());
            return false;
        }
        let mut is_open = true;
        let mut should_close = false;
        
        if !is_open {
            return false;
        }
        
        egui::Window::new(&self.title)
            .open(&mut is_open)
            .show(ctx, |ui| {
                ui.label(&self.message);
                ui.horizontal(|ui| {
                    if ui.button("确认").clicked() {
                        if let Some(on_confirm) = self.on_confirm.take() {
                            if let Err(e) = on_confirm(dialog_ctx.app) {
                                dialog_ctx.show_error(format!("操作失败: {}", e));
                            }
                        }
                        should_close = true;
                    }
                    if ui.button("取消").clicked() {
                        if let Some(on_cancel) = self.on_cancel.take() {
                            if let Err(e) = on_cancel(dialog_ctx.app) {
                                dialog_ctx.show_error(format!("操作失败: {}", e));
                            }
                        }
                        should_close = true;
                    }
                });
            });
            
        if should_close {
            is_open = false;
        }
        
        is_open
    }
    
    fn validate(&self) -> Result<()> {
        if self.title.is_empty() {
            return Err(TimeTrackerError::Dialog("Empty title".into()));
        }
        if self.message.is_empty() {
            return Err(TimeTrackerError::Dialog("Empty message".into()));
        }
        Ok(())
    }
}

impl std::fmt::Debug for ConfirmationDialog {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ConfirmationDialog")
            .field("title", &self.title)
            .field("message", &self.message)
            .field("on_confirm", &format_args!("<callback>"))
            .field("on_cancel", &format_args!("<callback>"))
            .finish()
    }
}

pub trait DialogHandler: std::any::Any + Send {
    fn show(&mut self, ctx: &egui::Context, dialog_ctx: &mut ui::DialogContext) -> bool;
}

impl DialogHandler for ConfirmationDialog {
    fn show(&mut self, ctx: &egui::Context, dialog_ctx: &mut ui::DialogContext) -> bool {
        if let Err(e) = self.validate() {
            dialog_ctx.show_error(e.to_string());
            return false;
        }
        
        let mut is_open = true;
        let mut should_close = false;
        
        egui::Window::new(&self.title)
            .open(&mut is_open)
            .show(ctx, |ui| {
                ui.label(&self.message);
                ui.horizontal(|ui| {
                    if ui.button("确认").clicked() {
                        if let Some(on_confirm) = self.on_confirm.take() {
                            if let Err(e) = on_confirm(dialog_ctx.app) {
                                dialog_ctx.show_error(e.to_string());
                            }
                        }
                        should_close = true;
                    }
                    if ui.button("取消").clicked() {
                        if let Some(on_cancel) = self.on_cancel.take() {
                            if let Err(e) = on_cancel(dialog_ctx.app) {
                                dialog_ctx.show_error(e.to_string());
                            }
                        }
                        should_close = true;
                    }
                });
            });
            
        if should_close {
            is_open = false;
        }
        
        is_open
    }
}

impl DialogHandler for ExportDialog {
    fn show(&mut self, ctx: &egui::Context, dialog_ctx: &mut ui::DialogContext) -> bool {
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
                            .add_filter("JSON", &["json"])
                            .save_file()
                        {
                            self.path = path.display().to_string();
                        }
                    }
                });

                // 日期范围
                ui.label("日期范围");
                ui.horizontal(|ui| {
                    // 显示当前选择的日期范围
                    ui.label(format!("从 {} 到 {}",
                        self.date_range.start.format("%Y-%m-%d"),
                        self.date_range.end.format("%Y-%m-%d")
                    ));
                    
                    if ui.button("选择日期范围").clicked() {
                        let start = self.date_range.start;
                        let end = self.date_range.end;
                        let date_picker = DateRangeDialog::new()
                            .with_dates(
                                start.date_naive(),
                                end.date_naive()
                            );

                        dialog_ctx.app.push_dialog(Box::new(date_picker));
                    }
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
                        let result = match self.format {
                            ExportFormat::CSV => self.export_csv(dialog_ctx.app),
                            ExportFormat::JSON => self.export_json(dialog_ctx.app),
                            ExportFormat::Excel => self.export_excel(dialog_ctx.app),
                        };

                        if let Err(e) = result {
                            dialog_ctx.show_error(format!("导出失败: {}", e));
                        }
                        dialog_ctx.pop_dialog();
                    }
                });
            });
        is_open
    }
}

impl DialogHandler for SettingsDialog {
    fn show(&mut self, ctx: &egui::Context, dialog_ctx: &mut ui::DialogContext) -> bool {
        let mut is_open = true;
        egui::Window::new("设置")
            .collapsible(false)
            .resizable(false)
            .open(&mut is_open)
            .show(ctx, |ui| {
                egui::ScrollArea::vertical().show(ui, |ui| {
                    self.show_general_settings(ui);
                    ui.add_space(styles::SPACING_LARGE);
                    self.show_pomodoro_settings(ui);
                    ui.add_space(styles::SPACING_LARGE);
                    self.show_notification_settings(ui);
                    ui.add_space(styles::SPACING_LARGE);
                    self.show_backup_settings(ui);
                    ui.add_space(styles::SPACING_LARGE);

                    // 按钮区域
                    ui.horizontal(|ui| {
                        if Button::new("取消")
                            .with_style(styles::ButtonStyle::outlined())
                            .show(ui)
                            .clicked()
                        {
                            dialog_ctx.pop_dialog();
                        }

                        if Button::new("保存")
                            .with_style(styles::ButtonStyle::primary())
                            .show(ui)
                            .clicked()
                        {
                            if let Err(e) = self.save_settings(dialog_ctx.app) {
                                dialog_ctx.show_error(format!("保存设置失败: {}", e));
                            } else {
                                dialog_ctx.pop_dialog();
                            }
                        }
                    });
                });
            });
        is_open
    }
}

pub struct DateRangeDialog {
    pub start_date: NaiveDate,
    pub end_date: NaiveDate,
    calendar_visible: bool,
    editing_start: bool,
}

impl DateRangeDialog {
    pub fn new() -> Self {
        Self {
            start_date: chrono::Local::now().date_naive(),
            end_date: chrono::Local::now().date_naive(),
            calendar_visible: false,
            editing_start: false,
        }
    }

    pub fn with_dates(mut self, start: NaiveDate, end: NaiveDate) -> Self {
        self.start_date = start;
        self.end_date = end;
        self
    }

    fn show_calendar(&mut self, ui: &mut egui::Ui, date: &mut NaiveDate) {
        ui.vertical(|ui| {
            // 显示年月选择器
            ui.horizontal(|ui| {
                if ui.button("◀").clicked() {
                    *date = date.checked_sub_months(chrono::Months::new(1)).unwrap();
                }
                ui.label(date.format("%Y年%m月").to_string());
                if ui.button("▶").clicked() {
                    *date = date.checked_add_months(chrono::Months::new(1)).unwrap();
                }
            });

            // 显示星期标题
            ui.horizontal(|ui| {
                for weekday in ["日", "一", "二", "三", "四", "五", "六"] {
                    ui.label(weekday);
                }
            });

            // 获取当月第一天和最后一天
            let first_day = NaiveDate::from_ymd_opt(date.year(), date.month(), 1).unwrap();
            let last_day = NaiveDate::from_ymd_opt(
                date.year(),
                date.month() + 1,
                1
            ).unwrap_or(
                NaiveDate::from_ymd_opt(date.year() + 1, 1, 1).unwrap()
            ).pred_opt().unwrap();

            // 计算第一天是星期几
            let first_weekday = first_day.weekday().num_days_from_sunday();

            // 显示日历网格
            let mut current_date = first_day - chrono::Duration::days(first_weekday as i64);
            while current_date <= last_day || current_date.weekday().num_days_from_sunday() != 0 {
                ui.horizontal(|ui| {
                    for _ in 0..7 {
                        let is_current_month = current_date.month() == date.month();
                        let is_selected = current_date == *date;

                        let text = current_date.day().to_string();
                        if ui.add(egui::Button::new(text)
                            .fill(if is_selected {
                                ui.style().visuals.selection.bg_fill
                            } else {
                                ui.style().visuals.widgets.noninteractive.bg_fill
                            })
                            .enabled(is_current_month)
                        ).clicked() {
                            *date = current_date;
                            self.calendar_visible = false;
                        }
                        current_date = current_date.succ_opt().unwrap();
                    }
                });
            }
        });
    }
}

impl DialogHandler for DateRangeDialog {
    fn show(&mut self, ctx: &egui::Context, dialog_ctx: &mut ui::DialogContext) -> bool {
        let mut is_open = true;
        let mut should_close = false;

        egui::Window::new("选择日期范围")
            .collapsible(false)
            .resizable(false)
            .default_width(300.0)
            .open(&mut is_open)
            .show(ctx, |ui| {
                ui.vertical(|ui| {
                    // 显示日期按钮
                    ui.horizontal(|ui| {
                        ui.label("开始日期:");
                        if ui.button(self.start_date.format("%Y-%m-%d").to_string()).clicked() {
                            self.calendar_visible = true;
                            self.editing_start = true;
                        }
                    });

                    ui.horizontal(|ui| {
                        ui.label("结束日期:");
                        if ui.button(self.end_date.format("%Y-%m-%d").to_string()).clicked() {
                            self.calendar_visible = true;
                            self.editing_start = false;
                        }
                    });

                    // 显示日历
                    if self.calendar_visible {
                        let mut temp_date = if self.editing_start {
                            self.start_date
                        } else {
                            self.end_date
                        };

                        self.show_calendar(ui, &mut temp_date);

                        if self.editing_start {
                            self.start_date = temp_date;
                        } else {
                            self.end_date = temp_date;
                        }
                    }

                    ui.add_space(8.0);

                    // 按钮区域
                    ui.horizontal(|ui| {
                        if ui.button("确定").clicked() {
                            // 先获取栈的长度
                            let stack_len = dialog_ctx.app.ui_state.dialog_stack.len();
                            if stack_len >= 2 {
                                // 然后使用索引访问
                                let export_dialog = dialog_ctx.app.ui_state.dialog_stack[stack_len - 2]
                                    .as_any_mut()
                                    .downcast_mut::<ExportDialog>();
                                
                                if let Some(export_dialog) = export_dialog {
                                    export_dialog.date_range.start = self.start_date
                                        .and_hms_opt(0, 0, 0)
                                        .unwrap()
                                        .and_local_timezone(chrono::Local)
                                        .unwrap();
                                    
                                    export_dialog.date_range.end = self.end_date
                                        .and_hms_opt(23, 59, 59)
                                        .unwrap()
                                        .and_local_timezone(chrono::Local)
                                        .unwrap();
                                }
                            }
                            should_close = true;
                        }
                        if ui.button("取消").clicked() {
                            should_close = true;
                        }
                    });
                });
            });

        if should_close {
            is_open = false;
        }

        is_open
    }
}

pub struct UpdateDialog {
    pub checking: bool,
    pub version: Option<String>,
    pub changelog: Option<String>,
    pub download_url: Option<String>,
}

impl UpdateDialog {
    pub fn new() -> Self {
        Self {
            checking: true,
            version: None,
            changelog: None,
            download_url: None,
        }
    }
}

pub struct AboutDialog {
    pub version: String,
    pub author: String,
    pub description: String,
}

impl AboutDialog {
    pub fn new() -> Self {
        Self {
            version: env!("CARGO_PKG_VERSION").to_string(),
            author: "Your Name".to_string(),
            description: "一个简单的时间跟踪工具".to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use eframe::egui::{Context, RawInput};
    use crate::ui::TimeTrackerApp;
    use crate::config::Config;
    use crate::storage::app_state::AppState;
    use tempfile::TempDir;
    use std::sync::{Arc, Mutex};
    use std::sync::mpsc;

    #[test]
    fn test_confirmation_dialog() {
        let ctx = Context::default();
        let temp_dir = TempDir::new().unwrap();
        
        // 创建配置
        let config = Arc::new(Mutex::new(Config::default()));
        
        // 创建应用状态
        let state = Arc::new(Mutex::new(AppState::new()));
        
        // 创建存储配置
        let storage_config = crate::storage::StorageConfig {
            data_dir: temp_dir.path().to_path_buf(),
            backup_enabled: false,
            backup_interval: std::time::Duration::from_secs(3600),
            max_backup_count: 1,
            vacuum_threshold: 1024,
        };
        
        let storage = Arc::new(Mutex::new(
            crate::storage::Storage::new(storage_config).unwrap()
        ));

        // 创建番茄钟计时器
        let pomodoro_timer = Arc::new(Mutex::new(
            crate::pomodoro::PomodoroTimer::new(
                Default::default(),
                crate::pomodoro::PomodoroCallbacks::default()
            )
        ));

        // 创建应用追踪器
        let app_tracker = Arc::new(Mutex::new(
            crate::app_tracker::AppTracker::new(Default::default()).unwrap()
        ));

        let app_state_manager = Arc::new(Mutex::new(
            crate::storage::app_state::AppStateManager::new(
                temp_dir.path().to_path_buf(),
                false
            ).unwrap()
        ));

        let (tray_sender, tray_receiver) = mpsc::channel();
        let tray_manager = Arc::new(Mutex::new(
            crate::tray::TrayManager::new(
                temp_dir.path().join("tray_icon.png"),
                tray_sender
            ).unwrap()
        ));

        let hotkey_manager = Arc::new(Mutex::new(
            crate::hotkeys::HotkeyManager::new(Default::default())
        ));

        let mut app = TimeTrackerApp::new(
            config.clone(),
            storage.clone(),
            pomodoro_timer,
            app_tracker,
            app_state_manager,
            tray_manager,
            hotkey_manager,
            tray_receiver,
        );

        let mut dialog = ConfirmationDialog {
            title: "Test".to_string(),
            message: "Test message".to_string(),
            on_confirm: None,
            on_cancel: None,
        };

        ctx.run(RawInput::default(), |ctx| {
            let config = config.lock().unwrap();
            let state = state.lock().unwrap();
            let storage = storage.lock().unwrap();
            
            let mut dialog_ctx = ui::DialogContext {
                app: &mut app,
                config: &config,
                state: &state,
                storage: &storage,
            };
            assert!(Dialog::show(&mut dialog, ctx, &mut dialog_ctx));
        });
    }
}

#[cfg(target_os = "windows")]
fn set_autostart(enabled: bool) -> Result<()> {
    crate::platform::windows::set_autostart(enabled)
}

#[cfg(not(target_os = "windows"))]
fn set_autostart(_enabled: bool) -> Result<()> {
    Ok(())
}