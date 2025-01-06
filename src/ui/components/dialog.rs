use super::Button;
use crate::error::Result;
use crate::ui::{styles, TimeTrackerApp, DialogHandler, DialogContext};
use eframe::egui;
use crate::error::TimeTrackerError;
use rfd::FileDialog;
use chrono::{NaiveDate, Local, Datelike};
use open;
use std::sync::{Arc, Mutex};

// 基础对话框特征
pub trait Dialog {
    fn show(&mut self, ctx: &egui::Context, dialog_ctx: &mut DialogContext) -> bool;
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
    fn show(&mut self, ctx: &egui::Context, dialog_ctx: &mut DialogContext) -> bool {
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
    fn show(&mut self, ctx: &egui::Context, dialog_ctx: &mut DialogContext) -> bool {
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
                    let mut date_picker = DateRangePicker::new();
                    let (start_date, end_date) = date_picker.show(ui);
                    
                    // 更新导出对话框的日期范围
                    self.date_range.start = start_date.and_hms_opt(0, 0, 0)
                        .unwrap()
                        .and_local_timezone(chrono::Local)
                        .unwrap();
                    
                    self.date_range.end = end_date.and_hms_opt(23, 59, 59)
                        .unwrap()
                        .and_local_timezone(chrono::Local)
                        .unwrap();
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
                    record.timestamp.format("%Y-%m-%d %H:%M:%S").to_string(),
                    record.app_name,
                    record.window_title,
                    record.duration.as_secs().to_string(),
                ])?;
            }
            
            writer.write_record(&[])?; // 空行分隔
        }

        // 导出番茄钟记录
        if self.include_pomodoros {
            writer.write_record(&["开始时间", "结束时间", "类型", "状态"])?;
            
            let records = app.storage.lock().unwrap()
                .get_pomodoro_records(self.date_range.start, self.date_range.end)?;
            
            for record in records {
                writer.write_record(&[
                    record.start_time.format("%Y-%m-%d %H:%M:%S").to_string(),
                    record.end_time.format("%Y-%m-%d %H:%M:%S").to_string(),
                    record.pomodoro_type.to_string(),
                    record.status.to_string(),
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
        use xlsxwriter::{Workbook, DateTime};

        let workbook = Workbook::new(&self.path);
        let storage = app.storage.lock().unwrap();

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
                sheet.write_datetime(row, 0, &DateTime::new(record.timestamp), None)?;
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
            sheet.write_string(0, 2, "类型", None)?;
            sheet.write_string(0, 3, "状态", None)?;

            let records = storage.get_pomodoro_records(
                self.date_range.start, 
                self.date_range.end
            )?;

            for (i, record) in records.iter().enumerate() {
                let row = (i + 1) as u32;
                sheet.write_datetime(row, 0, &DateTime::new(record.start_time), None)?;
                sheet.write_datetime(row, 1, &DateTime::new(record.end_time), None)?;
                sheet.write_string(row, 2, &record.pomodoro_type.to_string(), None)?;
                sheet.write_string(row, 3, &record.status.to_string(), None)?;
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
    fn show(&mut self, ctx: &egui::Context, dialog_ctx: &mut DialogContext) -> bool {
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
        if self.general_settings.autostart {
            crate::platform::windows::set_autostart(true)?;
        } else {
            crate::platform::windows::set_autostart(false)?;
        }
        
        // 重新配置番茄钟计时器
        let mut pomodoro_timer = app.pomodoro_timer.lock().unwrap();
        pomodoro_timer.update_config(config.pomodoro.clone());
        
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
    fn show(&mut self, ctx: &egui::Context, app: &mut TimeTrackerApp) -> bool {
        if let Err(e) = self.validate() {
            app.show_error(e.to_string());
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
                            if let Err(e) = on_confirm(app) {
                                app.show_error(format!("操作失败: {}", e));
                            }
                        }
                        should_close = true;
                    }
                    if ui.button("取消").clicked() {
                        if let Some(on_cancel) = self.on_cancel.take() {
                            if let Err(e) = on_cancel(app) {
                                app.show_error(format!("操作失败: {}", e));
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

impl DialogHandler for ConfirmationDialog {
    fn show(&mut self, ctx: &egui::Context, dialog_ctx: &mut DialogContext) -> bool {
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
                            should_close = true;
                        }
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
                    let mut date_picker = DateRangePicker::new();
                    let (start_date, end_date) = date_picker.show(ui);
                    
                    // 更新导出对话框的日期范围
                    self.date_range.start = start_date.and_hms_opt(0, 0, 0)
                        .unwrap()
                        .and_local_timezone(chrono::Local)
                        .unwrap();
                    
                    self.date_range.end = end_date.and_hms_opt(23, 59, 59)
                        .unwrap()
                        .and_local_timezone(chrono::Local)
                        .unwrap();
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
    fn show(&mut self, ctx: &egui::Context, dialog_ctx: &mut DialogContext) -> bool {
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

pub struct DateRangePicker {
    start_date: NaiveDate,
    end_date: NaiveDate,
    start_popup_id: egui::Id,
    end_popup_id: egui::Id,
    calendar_visible: bool,
}

impl DateRangePicker {
    pub fn new() -> Self {
        let today = Local::now().date_naive();
        Self {
            start_date: today - chrono::Duration::days(7),  // 默认显示最近7天
            end_date: today,
            start_popup_id: egui::Id::new("start_date_popup"),
            end_popup_id: egui::Id::new("end_date_popup"),
            calendar_visible: false,
        }
    }

    pub fn show(&mut self, ui: &mut egui::Ui) -> (NaiveDate, NaiveDate) {
        ui.horizontal(|ui| {
            // 开始日期
            ui.label("从：");
            let mut start_date = self.start_date;
            let start_clicked = self.date_picker_button(ui, &mut start_date, self.start_popup_id);
            self.start_date = start_date;
            if start_clicked {
                self.calendar_visible = !self.calendar_visible;
            }

            ui.add_space(8.0);

            // 结束日期
            ui.label("至：");
            let mut end_date = self.end_date;
            let end_clicked = self.date_picker_button(ui, &mut end_date, self.end_popup_id);
            self.end_date = end_date;
            if end_clicked {
                self.calendar_visible = !self.calendar_visible;
            }

            ui.add_space(16.0);

            // 快捷选择按钮
            if ui.small_button("今天").clicked() {
                let today = Local::now().date_naive();
                self.start_date = today;
                self.end_date = today;
            }
            
            ui.add_space(4.0);
            
            if ui.small_button("最近7天").clicked() {
                self.end_date = Local::now().date_naive();
                self.start_date = self.end_date - chrono::Duration::days(7);
            }
            
            ui.add_space(4.0);
            
            if ui.small_button("最近30天").clicked() {
                self.end_date = Local::now().date_naive();
                self.start_date = self.end_date - chrono::Duration::days(30);
            }
        });

        (self.start_date, self.end_date)
    }

    fn date_picker_button(&mut self, ui: &mut egui::Ui, date: &mut NaiveDate, popup_id: egui::Id) -> bool {
        let mut clicked = false;
        let button_response = ui.add(egui::Button::new(date.format("%Y-%m-%d").to_string()));
        
        if button_response.clicked() {
            clicked = true;
        }

        if self.calendar_visible {
            let popup = egui::popup::popup_below_widget(ui, popup_id, &button_response, |ui| {
                self.show_calendar(ui, date);
            });
            
            if popup.clicked_elsewhere() {
                self.calendar_visible = false;
            }
        }

        clicked
    }

    fn show_calendar(&mut self, ui: &mut egui::Ui, selected_date: &mut NaiveDate) {
        let mut year = selected_date.year();
        let mut month = selected_date.month() as i32;

        ui.horizontal(|ui| {
            if ui.small_button("◀").clicked() {
                month -= 1;
                if month < 1 {
                    month = 12;
                    year -= 1;
                }
            }
            
            ui.label(format!("{:04}-{:02}", year, month));
            
            if ui.small_button("▶").clicked() {
                month += 1;
                if month > 12 {
                    month = 1;
                    year += 1;
                }
            }
        });

        ui.add_space(4.0);

        // 显示星期标题
        ui.horizontal(|ui| {
            for weekday in ["日", "一", "二", "三", "四", "五", "六"] {
                ui.label(weekday);
            }
        });

        // 计算当月第一天是星期几
        let first_day = NaiveDate::from_ymd_opt(year, month as u32, 1).unwrap();
        let mut current_day = first_day - chrono::Duration::days(first_day.weekday().num_days_from_sunday() as i64);

        // 显示日历网格
        for _week in 0..6 {
            ui.horizontal(|ui| {
                for _weekday in 0..7 {
                    let is_current_month = current_day.month() == month as u32;
                    let is_selected = current_day == *selected_date;
                    
                    let mut button = egui::Button::new(format!("{:2}", current_day.day()));
                    
                    if is_selected {
                        button = button.fill(ui.style().visuals.selection.bg_fill);
                    } else if !is_current_month {
                        button = button.text_color(ui.style().visuals.weak_text_color());
                    }
                    
                    if ui.add(button).clicked() {
                        *selected_date = current_day;
                        self.calendar_visible = false;
                    }
                    
                    current_day += chrono::Duration::days(1);
                }
            });
        }
    }
}

// 关于对话框
pub struct AboutDialog;

impl AboutDialog {
    pub fn new() -> Self {
        Self
    }
}

impl Dialog for AboutDialog {
    fn show(&mut self, ctx: &egui::Context, app: &mut TimeTrackerApp) -> bool {
        let mut is_open = true;
        egui::Window::new("关于")
            .collapsible(false)
            .resizable(false)
            .default_width(400.0)
            .open(&mut is_open)
            .show(ctx, |ui| {
                ui.vertical_centered(|ui| {
                    // 应用名称和版本
                    ui.heading("时间追踪器");
                    ui.label(format!("版本 {}", env!("CARGO_PKG_VERSION")));
                    
                    ui.add_space(16.0);
                    
                    // 应用描述
                    ui.label("一个简单的时间追踪工具，帮助你更好地管理时间。");
                    
                    ui.add_space(8.0);
                    
                    // 版权信息
                    ui.label("© 2024 TimeTracker Team");
                    
                    ui.add_space(16.0);
                    
                    // 链接
                    ui.horizontal(|ui| {
                        if ui.link("项目主页").clicked() {
                            if let Err(e) = open::that("https://github.com/your/timetracker") {
                                app.show_error(format!("无法打开链接: {}", e));
                            }
                        }
                        ui.label(" | ");
                        if ui.link("报告问题").clicked() {
                            if let Err(e) = open::that("https://github.com/your/timetracker/issues") {
                                app.show_error(format!("无法打开链接: {}", e));
                            }
                        }
                    });
                });
            });
        is_open
    }
}

// 更新检查对话框
pub struct UpdateDialog {
    checking: bool,
    update_info: Option<UpdateInfo>,
    error: Option<String>,
}

#[derive(Debug)]
struct UpdateInfo {
    version: String,
    release_date: String,
    download_url: String,
    release_notes: String,
}

impl UpdateDialog {
    pub fn new() -> Self {
        Self {
            checking: true,
            update_info: None,
            error: None,
        }
    }

    async fn check_update() -> Result<Option<UpdateInfo>> {
        use reqwest;
        use semver::Version;
        use serde_json::Value;

        // 获取当前版本
        let current_version = Version::parse(env!("CARGO_PKG_VERSION"))?;

        // 从 GitHub API 获取最新版本信息
        let client = reqwest::Client::new();
        let response = client
            .get("https://api.github.com/repos/your/timetracker/releases/latest")
            .header("User-Agent", "TimeTracker")
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(TimeTrackerError::Platform(format!(
                "GitHub API 请求失败: {}",
                response.status()
            )));
        }

        let release: Value = response.json().await?;
        
        // 解析版本信息
        let latest_version = release["tag_name"]
            .as_str()
            .ok_or_else(|| TimeTrackerError::Platform("无效的版本标签".into()))?
            .trim_start_matches('v');
        
        let latest_version = Version::parse(latest_version)?;

        // 比较版本
        if latest_version <= current_version {
            return Ok(None);
        }

        // 有新版本，返回更新信息
        Ok(Some(UpdateInfo {
            version: latest_version.to_string(),
            release_date: release["published_at"]
                .as_str()
                .map(|d| {
                    chrono::DateTime::parse_from_rfc3339(d)
                        .map(|dt| dt.format("%Y-%m-%d").to_string())
                        .unwrap_or_else(|_| "未知日期".to_string())
                })
                .unwrap_or_else(|| "未知日期".to_string()),
            download_url: release["html_url"]
                .as_str()
                .unwrap_or("https://github.com/your/timetracker/releases")
                .to_string(),
            release_notes: release["body"]
                .as_str()
                .unwrap_or("暂无更新说明")
                .to_string(),
        }))
    }
}

impl Dialog for UpdateDialog {
    fn show(&mut self, ctx: &egui::Context, app: &mut TimeTrackerApp) -> bool {
        let mut is_open = true;
        egui::Window::new("检查更新")
            .collapsible(false)
            .resizable(false)
            .default_width(400.0)
            .open(&mut is_open)
            .show(ctx, |ui| {
                if self.checking {
                    ui.vertical_centered(|ui| {
                        ui.add_space(20.0);
                        ui.spinner();
                        ui.label("正在检查更新...");
                        ui.add_space(20.0);
                    });
                    
                    // 启动异步检查
                    let ctx = ctx.clone();
                    let update_info = Arc::new(Mutex::new(self.update_info.take()));
                    let error = Arc::new(Mutex::new(self.error.take()));
                    
                    app.spawn_task(Box::pin(async move {
                        match Self::check_update().await {
                            Ok(info) => {
                                *update_info.lock().unwrap() = info;
                                ctx.request_repaint();
                            }
                            Err(e) => {
                                *error.lock().unwrap() = Some(e.to_string());
                                ctx.request_repaint();
                            }
                        }
                    }));
                    
                    self.checking = false;
                } else if let Some(error) = &self.error {
                    ui.vertical_centered(|ui| {
                        ui.label("检查更新失败");
                        ui.label(error);
                        if ui.button("关闭").clicked() {
                            app.pop_dialog();
                        }
                    });
                } else if let Some(update_info) = &self.update_info {
                    ui.vertical_centered(|ui| {
                        ui.heading("发现新版本");
                        ui.label(format!("版本: {}", update_info.version));
                        ui.label(format!("发布日期: {}", update_info.release_date));
                        
                        ui.add_space(8.0);
                        
                        ui.group(|ui| {
                            ui.label("更新内容:");
                            ui.label(&update_info.release_notes);
                        });
                        
                        ui.add_space(16.0);
                        
                        ui.horizontal(|ui| {
                            if ui.button("下载更新").clicked() {
                                if let Err(e) = open::that(&update_info.download_url) {
                                    app.show_error(format!("无法打开下载链接: {}", e));
                                }
                                app.pop_dialog();
                            }
                            if ui.button("稍后提醒").clicked() {
                                app.pop_dialog();
                            }
                        });
                    });
                } else {
                    ui.vertical_centered(|ui| {
                        ui.label("您使用的已经是最新版本");
                        if ui.button("确定").clicked() {
                            app.pop_dialog();
                        }
                    });
                }
            });
        is_open
    }
}

pub struct DateRangeDialog {
    pub open: bool,
    pub start_date: NaiveDate,
    pub end_date: NaiveDate,
    pub on_close: Option<Box<dyn FnOnce(Option<(NaiveDate, NaiveDate)>)>>,
}

impl Default for DateRangeDialog {
    fn default() -> Self {
        let today = Local::now().date_naive();
        Self {
            open: false,
            start_date: today,
            end_date: today,
            on_close: None,
        }
    }
}

impl DateRangeDialog {
    pub fn show(&mut self, ctx: &egui::Context) {
        if !self.open {
            return;
        }

        egui::Window::new("选择日期范围")
            .collapsible(false)
            .resizable(false)
            .show(ctx, |ui| {
                // 开始日期
                ui.horizontal(|ui| {
                    ui.label("开始日期:");
                    let mut start_date = self.start_date;
                    if let Some(date) = date_picker(ui, &mut start_date) {
                        self.start_date = date;
                    }
                });

                // 结束日期
                ui.horizontal(|ui| {
                    ui.label("结束日期:");
                    let mut end_date = self.end_date;
                    if let Some(date) = date_picker(ui, &mut end_date) {
                        self.end_date = date;
                    }
                });

                ui.separator();

                ui.horizontal(|ui| {
                    if ui.button("确定").clicked() {
                        if let Some(on_close) = self.on_close.take() {
                            on_close(Some((self.start_date, self.end_date)));
                        }
                        self.open = false;
                    }

                    if ui.button("取消").clicked() {
                        if let Some(on_close) = self.on_close.take() {
                            on_close(None);
                        }
                        self.open = false;
                    }
                });
            });
    }
}

fn date_picker(ui: &mut egui::Ui, date: &mut NaiveDate) -> Option<NaiveDate> {
    let mut changed = None;
    
    ui.horizontal(|ui| {
        let mut year = date.year();
        let mut month = date.month() as i32;
        let mut day = date.day() as i32;

        ui.add(egui::DragValue::new(&mut year).clamp_range(2000..=2100));
        ui.label("-");
        ui.add(egui::DragValue::new(&mut month).clamp_range(1..=12));
        ui.label("-");
        ui.add(egui::DragValue::new(&mut day).clamp_range(1..=31));

        if let Some(new_date) = NaiveDate::from_ymd_opt(year, month as u32, day as u32) {
            if new_date != *date {
                changed = Some(new_date);
            }
        }
    });

    changed
}

impl DialogHandler for AboutDialog {
    fn show(&mut self, ctx: &egui::Context, _dialog_ctx: &mut DialogContext) -> bool {
        let mut is_open = true;
        egui::Window::new("关于")
            .collapsible(false)
            .resizable(false)
            .open(&mut is_open)
            .show(ctx, |ui| {
                ui.label("Time Tracker");
                ui.label("版本: 0.1.0");
                ui.label("作者: Your Name");
                ui.separator();
                ui.label("一个简单的时间跟踪工具");
                if ui.button("关闭").clicked() {
                    is_open = false;
                }
            });
        is_open
    }
}

impl DialogHandler for UpdateDialog {
    fn show(&mut self, ctx: &egui::Context, _dialog_ctx: &mut DialogContext) -> bool {
        egui::Window::new("检查更新")
            .collapsible(false)
            .resizable(false)
            .show(ctx, |ui| {
                ui.label("正在检查更新...");
                if ui.button("关闭").clicked() {
                    is_open = false;
                }
            });
        is_open
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use eframe::egui::{Context, RawInput};
    use crate::ui::TimeTrackerApp;
    use tempfile::TempDir;
    use std::sync::{Arc, Mutex};
    use std::sync::mpsc;

    #[test]
    fn test_confirmation_dialog() {
        let ctx = Context::default();
        let temp_dir = TempDir::new().unwrap();
        
        // 创建所需的组件
        let config = Arc::new(Mutex::new(Default::default()));
        
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
        let (tray_sender, tray_event_receiver) = mpsc::channel();
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
            config,
            storage,
            pomodoro_timer,
            app_tracker,
            app_state_manager,
            tray_manager,
            hotkey_manager,
            tray_event_receiver,
        );

        let mut dialog = ConfirmationDialog {
            title: "Test".to_string(),
            message: "Test message".to_string(),
            on_confirm: None,
            on_cancel: None,
        };

        ctx.run(RawInput::default(), |ctx| {
            assert!(Dialog::show(&mut dialog, ctx, &mut app));
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