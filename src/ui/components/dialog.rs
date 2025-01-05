// src/ui/components/dialog.rs

use super::Button;
use crate::error::Result;
use crate::ui::{styles, TimeTrackerApp, DialogHandler, DialogContext};
use eframe::egui;
use crate::error::TimeTrackerError;
use rfd::FileDialog;

// 基础对话框特征
pub trait Dialog {
    fn show(&mut self, ctx: &egui::Context, app: &mut TimeTrackerApp) -> bool;
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
}

impl Dialog for ProjectDialog {
    fn show(&mut self, ctx: &egui::Context, app: &mut TimeTrackerApp) -> bool {
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
                        app.pop_dialog();
                    }

                    if Button::new("保存")
                        .enabled(!self.name.is_empty())
                        .show(ui)
                        .clicked()
                    {
                        if let Some(on_save) = self.on_save.take() {
                            if let Err(e) = on_save(
                                app,
                                self.name.clone(),
                                self.description.clone(),
                                self.color,
                            ) {
                                app.show_error(e.to_string());
                            }
                        }
                        app.pop_dialog();
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
    fn show(&mut self, ctx: &egui::Context, app: &mut TimeTrackerApp) -> bool {
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
                        app.pop_dialog();
                    }

                    if Button::new("保存")
                        .enabled(!self.name.is_empty())
                        .show(ui)
                        .clicked()
                    {
                        if let Some(on_save) = self.on_save.take() {
                            if let Err(e) = on_save(app, self.name.clone(), self.color) {
                                app.show_error(e.to_string());
                            }
                        }
                        app.pop_dialog();
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
    fn show(&mut self, ctx: &egui::Context, app: &mut TimeTrackerApp) -> bool {
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
                    // TODO: 添加日期选择器
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
                        app.pop_dialog();
                    }

                    if Button::new("导出")
                        .enabled(!self.path.is_empty())
                        .show(ui)
                        .clicked()
                    {
                        self.export_data(app);
                        app.pop_dialog();
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

    fn export_csv(&self, _app: &mut TimeTrackerApp) -> Result<()> {
        // TODO: 实现CSV导出
        Ok(())
    }

    fn export_json(&self, _app: &mut TimeTrackerApp) -> Result<()> {
        // TODO: 实现JSON导出
        Ok(())
    }

    fn export_excel(&self, _app: &mut TimeTrackerApp) -> Result<()> {
        // TODO: 实现Excel导出
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
    fn show(&mut self, ctx: &egui::Context, app: &mut TimeTrackerApp) -> bool {
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
                            app.pop_dialog();
                        }

                        if Button::new("保存")
                            .with_style(styles::ButtonStyle::primary())
                            .show(ui)
                            .clicked()
                        {
                            self.save_settings(app);
                            app.pop_dialog();
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

    fn save_settings(&self, _app: &mut TimeTrackerApp) {
        // TODO: 实现设置保存
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
    fn show(&mut self, ctx: &egui::Context, _dialog_ctx: &mut DialogContext) -> bool {
        if let Err(e) = self.validate() {
            // TODO: 添加错误处理
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
                            // TODO: 使用 dialog_ctx 而不是 app
                            should_close = true;
                        }
                    }
                    if ui.button("取消").clicked() {
                        if let Some(on_cancel) = self.on_cancel.take() {
                            // TODO: 使用 dialog_ctx 而不是 app
                            should_close = true;
                        }
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
    fn show(&mut self, ctx: &egui::Context, _dialog_ctx: &mut DialogContext) -> bool {
        let mut is_open = true;
        let mut should_close = false;

        egui::Window::new("导出数据")
            .open(&mut is_open)
            .show(ctx, |ui| {
                // 实现导出对话框的显示逻辑
                if ui.button("导出").clicked() {
                    // 处理导出逻辑
                    should_close = true;
                }
            });

        if should_close {
            is_open = false;
        }
        is_open
    }
}

impl DialogHandler for SettingsDialog {
    fn show(&mut self, ctx: &egui::Context, _dialog_ctx: &mut DialogContext) -> bool {
        let mut is_open = true;
        let mut should_close = false;

        egui::Window::new("设置")
            .open(&mut is_open)
            .show(ctx, |ui| {
                // 实现设置对话框的显示逻辑
                if ui.button("保存").clicked() {
                    // 处理保存逻辑
                    should_close = true;
                }
            });

        if should_close {
            is_open = false;
        }
        is_open
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use eframe::egui::Context;
    use crate::ui::TimeTrackerApp;
    use tempfile::TempDir;

    #[test]
    fn test_confirmation_dialog() {
        let ctx = Context::default();
        let temp_dir = TempDir::new().unwrap();
        let mut app = TimeTrackerApp::new(
            Default::default(),
            crate::storage::Storage::new(&temp_dir.path().to_path_buf()).unwrap(),
            Default::default(),
            Default::default(),
            Default::default(),
        );

        let mut dialog = ConfirmationDialog {
            title: "Test".to_string(),
            message: "Test message".to_string(),
            on_confirm: None,
            on_cancel: None,
        };

        ctx.run(|ctx| {
            assert!(dialog.show(ctx, &mut app));
        });
    }
}