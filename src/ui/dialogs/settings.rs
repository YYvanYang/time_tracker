use super::base::{Dialog, DialogContext};
use crate::config::*;
use crate::error::Result;
use eframe::egui;
use crate::ui::components::Button;
use crate::ui::styles;
use rfd::FileDialog;
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct GeneralSettings {
    pub autostart: bool,
    pub minimize_to_tray: bool,
    pub check_updates: bool,
    pub language: String,
}

#[derive(Debug, Clone)]
pub struct PomodoroSettings {
    pub work_duration: std::time::Duration,
    pub short_break_duration: std::time::Duration,
    pub long_break_duration: std::time::Duration,
    pub long_break_interval: u32,
    pub auto_start_breaks: bool,
    pub auto_start_pomodoros: bool,
}

#[derive(Debug, Clone)]
pub struct NotificationSettings {
    pub enabled: bool,
    pub sound_enabled: bool,
    pub sound_volume: u8,
}

#[derive(Debug, Clone)]
pub struct BackupSettings {
    pub enabled: bool,
    pub interval: u32,
    pub max_backup_count: u32,
    pub backup_path: String,
}

pub struct SettingsDialog {
    pub general_settings: GeneralSettings,
    pub pomodoro_settings: PomodoroSettings,
    pub notification_settings: NotificationSettings,
    pub backup_settings: BackupSettings,
}

impl SettingsDialog {
    pub fn new(config: &Config) -> Self {
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
                ui.label("保留备份数量");
                ui.add(egui::DragValue::new(&mut self.backup_settings.max_backup_count)
                    .clamp_range(1..=100));
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

    fn save_settings(&self, dialog_ctx: &mut DialogContext) -> Result<()> {
        let mut config = dialog_ctx.config.clone();
        
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
        config.storage.data_dir = PathBuf::from(&self.backup_settings.backup_path);
        
        dialog_ctx.app.save_config()?;

        Ok(())
    }
}

impl Dialog for SettingsDialog {
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
                            if let Err(e) = self.save_settings(dialog_ctx) {
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