use iced::{
    widget::{Button, Column, Container, Row, Text, TextInput, Slider, Checkbox, PickList},
    Element, Length,
};
use crate::infrastructure::config::Config;
use crate::presentation::ui::styles;
use super::base::{Dialog, DialogContainer};
use std::path::PathBuf;
use std::time::Duration;

#[derive(Debug, Clone)]
pub enum SettingsMessage {
    AutostartChanged(bool),
    MinimizeToTrayChanged(bool),
    LanguageChanged(String),
    WorkDurationChanged(String),
    ShortBreakDurationChanged(String),
    LongBreakDurationChanged(String),
    LongBreakIntervalChanged(String),
    AutoStartBreaksChanged(bool),
    AutoStartPomodorosChanged(bool),
    SoundEnabledChanged(bool),
    SoundVolumeChanged(i32),
    BackupEnabledChanged(bool),
    BackupIntervalChanged(String),
    MaxBackupCountChanged(String),
    BackupPathChanged(String),
    BrowseBackupPath,
    SaveSettings,
    NoOp,
}

#[derive(Debug, Clone)]
pub struct GeneralSettings {
    pub autostart: bool,
    pub minimize_to_tray: bool,
    pub check_updates: bool,
    pub language: String,
}

#[derive(Debug, Clone)]
pub struct PomodoroSettings {
    pub work_duration: Duration,
    pub short_break_duration: Duration,
    pub long_break_duration: Duration,
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
    general_settings: GeneralSettings,
    pomodoro_settings: PomodoroSettings,
    notification_settings: NotificationSettings,
    backup_settings: BackupSettings,
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
}

impl Dialog for SettingsDialog {
    fn view(&self) -> Element<SettingsMessage> {
        let content = Column::new()
            .spacing(20)
            .push(Text::new("Settings").size(24))
            .push(self.view_general_settings())
            .push(self.view_pomodoro_settings())
            .push(self.view_notification_settings())
            .push(self.view_backup_settings())
            .push(
                Row::new()
                    .spacing(10)
                    .push(
                        Button::new(Text::new("Cancel"))
                            .style(styles::button::primary()),
                    )
                    .push(
                        Button::new(Text::new("Save"))
                            .style(styles::button::primary()),
                    ),
            );

        DialogContainer::new()
            .push(content)
            .spacing(20)
            .into_element()
    }

    fn update(&mut self, message: SettingsMessage) {
        // TODO: 实现更新逻辑
    }
}

impl SettingsDialog {
    fn view_general_settings(&self) -> Element<SettingsMessage> {
        Column::new()
            .spacing(10)
            .push(Text::new("General Settings").size(20))
            .push(
                Checkbox::new(
                    "Auto Start",
                    self.general_settings.autostart,
                    SettingsMessage::AutostartChanged,
                ),
            )
            .push(
                Checkbox::new(
                    "Minimize to Tray",
                    self.general_settings.minimize_to_tray,
                    SettingsMessage::MinimizeToTrayChanged,
                ),
            )
            .push(
                Row::new()
                    .spacing(10)
                    .push(Text::new("Language"))
                    .push(
                        PickList::new(
                            &["zh-CN", "en"],
                            Some(&self.general_settings.language),
                            SettingsMessage::LanguageChanged,
                        )
                        .width(Length::Fill),
                    ),
            )
            .into()
    }

    fn view_pomodoro_settings(&self) -> Element<SettingsMessage> {
        Column::new()
            .spacing(10)
            .push(Text::new("Pomodoro Settings").size(20))
            .push(
                Row::new()
                    .spacing(10)
                    .push(Text::new("Work Duration (minutes)"))
                    .push(
                        TextInput::new(
                            "Enter work duration",
                            &(self.pomodoro_settings.work_duration.as_secs() / 60).to_string(),
                        )
                        .on_input(SettingsMessage::WorkDurationChanged)
                        .width(Length::Units(60)),
                    ),
            )
            .push(
                Row::new()
                    .spacing(10)
                    .push(Text::new("Short Break Duration (minutes)"))
                    .push(
                        TextInput::new(
                            "Enter short break duration",
                            &(self.pomodoro_settings.short_break_duration.as_secs() / 60).to_string(),
                        )
                        .on_input(SettingsMessage::ShortBreakDurationChanged)
                        .width(Length::Units(60)),
                    ),
            )
            .push(
                Row::new()
                    .spacing(10)
                    .push(Text::new("Long Break Duration (minutes)"))
                    .push(
                        TextInput::new(
                            "Enter long break duration",
                            &(self.pomodoro_settings.long_break_duration.as_secs() / 60).to_string(),
                        )
                        .on_input(SettingsMessage::LongBreakDurationChanged)
                        .width(Length::Units(60)),
                    ),
            )
            .push(
                Row::new()
                    .spacing(10)
                    .push(Text::new("Long Break Interval"))
                    .push(
                        TextInput::new(
                            "Enter interval",
                            &self.pomodoro_settings.long_break_interval.to_string(),
                        )
                        .on_input(SettingsMessage::LongBreakIntervalChanged)
                        .width(Length::Units(60)),
                    ),
            )
            .push(
                Checkbox::new(
                    "Auto Start Breaks",
                    self.pomodoro_settings.auto_start_breaks,
                    SettingsMessage::AutoStartBreaksChanged,
                ),
            )
            .push(
                Checkbox::new(
                    "Auto Start Pomodoros",
                    self.pomodoro_settings.auto_start_pomodoros,
                    SettingsMessage::AutoStartPomodorosChanged,
                ),
            )
            .into()
    }

    fn view_notification_settings(&self) -> Element<SettingsMessage> {
        Column::new()
            .spacing(10)
            .push(Text::new("Notification Settings").size(20))
            .push(
                Checkbox::new(
                    "Enable Notifications",
                    self.notification_settings.enabled,
                    SettingsMessage::SoundEnabledChanged,
                ),
            )
            .push(
                Checkbox::new(
                    "Enable Sound",
                    self.notification_settings.sound_enabled,
                    SettingsMessage::SoundEnabledChanged,
                ),
            )
            .push(
                Row::new()
                    .spacing(10)
                    .push(Text::new("Volume"))
                    .push(
                        Slider::new(
                            0..=100,
                            self.notification_settings.sound_volume as i32,
                            SettingsMessage::SoundVolumeChanged,
                        )
                        .width(Length::Fill),
                    ),
            )
            .into()
    }

    fn view_backup_settings(&self) -> Element<SettingsMessage> {
        Column::new()
            .spacing(10)
            .push(Text::new("Backup Settings").size(20))
            .push(
                Checkbox::new(
                    "Enable Auto Backup",
                    self.backup_settings.enabled,
                    SettingsMessage::BackupEnabledChanged,
                ),
            )
            .push(
                Row::new()
                    .spacing(10)
                    .push(Text::new("Backup Interval (hours)"))
                    .push(
                        TextInput::new(
                            "Enter interval",
                            &self.backup_settings.interval.to_string(),
                        )
                        .on_input(SettingsMessage::BackupIntervalChanged)
                        .width(Length::Units(60)),
                    ),
            )
            .push(
                Row::new()
                    .spacing(10)
                    .push(Text::new("Max Backup Count"))
                    .push(
                        TextInput::new(
                            "Enter count",
                            &self.backup_settings.max_backup_count.to_string(),
                        )
                        .on_input(SettingsMessage::MaxBackupCountChanged)
                        .width(Length::Units(60)),
                    ),
            )
            .push(
                Row::new()
                    .spacing(10)
                    .push(Text::new("Backup Path"))
                    .push(
                        TextInput::new(
                            "Enter path",
                            &self.backup_settings.backup_path,
                        )
                        .on_input(SettingsMessage::BackupPathChanged)
                        .width(Length::Fill),
                    )
                    .push(
                        Button::new(Text::new("Browse"))
                            .style(styles::button::primary())
                            .on_press(SettingsMessage::BrowseBackupPath),
                    ),
            )
            .into()
    }
} 