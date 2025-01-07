use crate::application::App;
use crate::infrastructure::config::{Config, Theme};
use crate::presentation::state::SharedState;
use iced::{
    widget::{Button, Checkbox, Column, Container, Row, Text, TextInput, PickList},
    Element, Length, Command,
};
use std::sync::Arc;
use std::time::Duration;

#[derive(Debug, Clone)]
pub enum Message {
    AutostartChanged(bool),
    MinimizeToTrayChanged(bool),
    LanguageChanged(String),
    ThemeChanged(Theme),
    WorkDurationChanged(String),
    ShortBreakDurationChanged(String),
    LongBreakDurationChanged(String),
    LongBreakIntervalChanged(String),
    AutoStartBreaksChanged(bool),
    AutoStartPomodorosChanged(bool),
    SoundEnabledChanged(bool),
    SoundVolumeChanged(String),
    BackupEnabledChanged(bool),
    BackupIntervalChanged(String),
    MaxBackupCountChanged(String),
    SaveSettings,
    SettingsSaved,
}

pub struct SettingsView {
    app: Arc<App>,
    state: SharedState,
    config: Config,
}

impl SettingsView {
    pub fn new(app: Arc<App>, state: SharedState) -> Self {
        let config = app.command_handler().get_config().clone();
        Self {
            app,
            state,
            config,
        }
    }

    pub fn update(&mut self, message: Message) -> Command<Message> {
        match message {
            Message::AutostartChanged(enabled) => {
                self.config.general.autostart = enabled;
                Command::none()
            }
            Message::MinimizeToTrayChanged(enabled) => {
                self.config.general.minimize_to_tray = enabled;
                Command::none()
            }
            Message::LanguageChanged(language) => {
                self.config.general.language = language;
                Command::none()
            }
            Message::ThemeChanged(theme) => {
                self.config.ui.theme = theme;
                Command::none()
            }
            Message::WorkDurationChanged(duration) => {
                if let Ok(minutes) = duration.parse::<u64>() {
                    self.config.pomodoro.work_duration = Duration::from_secs(minutes * 60);
                }
                Command::none()
            }
            Message::ShortBreakDurationChanged(duration) => {
                if let Ok(minutes) = duration.parse::<u64>() {
                    self.config.pomodoro.short_break_duration = Duration::from_secs(minutes * 60);
                }
                Command::none()
            }
            Message::LongBreakDurationChanged(duration) => {
                if let Ok(minutes) = duration.parse::<u64>() {
                    self.config.pomodoro.long_break_duration = Duration::from_secs(minutes * 60);
                }
                Command::none()
            }
            Message::LongBreakIntervalChanged(interval) => {
                if let Ok(value) = interval.parse::<u32>() {
                    self.config.pomodoro.long_break_interval = value;
                }
                Command::none()
            }
            Message::AutoStartBreaksChanged(enabled) => {
                self.config.pomodoro.auto_start_breaks = enabled;
                Command::none()
            }
            Message::AutoStartPomodorosChanged(enabled) => {
                self.config.pomodoro.auto_start_pomodoros = enabled;
                Command::none()
            }
            Message::SoundEnabledChanged(enabled) => {
                self.config.pomodoro.sound_enabled = enabled;
                Command::none()
            }
            Message::SoundVolumeChanged(volume) => {
                if let Ok(value) = volume.parse::<u8>() {
                    self.config.pomodoro.sound_volume = value;
                }
                Command::none()
            }
            Message::BackupEnabledChanged(enabled) => {
                self.config.storage.backup_enabled = enabled;
                Command::none()
            }
            Message::BackupIntervalChanged(interval) => {
                if let Ok(hours) = interval.parse::<u64>() {
                    self.config.storage.backup_interval = Duration::from_secs(hours * 3600);
                }
                Command::none()
            }
            Message::MaxBackupCountChanged(count) => {
                if let Ok(value) = count.parse::<u32>() {
                    self.config.storage.max_backup_count = value;
                }
                Command::none()
            }
            Message::SaveSettings => {
                let config = self.config.clone();
                let app = self.app.clone();
                
                Command::perform(
                    async move {
                        app.command_handler().update_config().await.ok();
                    },
                    |_| Message::SettingsSaved,
                )
            }
            Message::SettingsSaved => Command::none(),
        }
    }

    pub fn view(&self) -> Element<Message> {
        let general_settings = self.general_settings_view();
        let pomodoro_settings = self.pomodoro_settings_view();
        let backup_settings = self.backup_settings_view();

        let save_button = Button::new(Text::new("保存设置"))
            .on_press(Message::SaveSettings)
            .padding(10);

        let content = Column::new()
            .push(general_settings)
            .push(pomodoro_settings)
            .push(backup_settings)
            .push(save_button)
            .spacing(20)
            .padding(20);

        Container::new(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }

    fn general_settings_view(&self) -> Element<Message> {
        let title = Text::new("常规设置").size(20);

        let autostart = Checkbox::new(
            "开机自启动",
            self.config.general.autostart,
            Message::AutostartChanged,
        );

        let minimize_to_tray = Checkbox::new(
            "最小化到托盘",
            self.config.general.minimize_to_tray,
            Message::MinimizeToTrayChanged,
        );

        let language = TextInput::new(
            "语言",
            &self.config.general.language,
        )
        .on_input(Message::LanguageChanged)
        .padding(10);

        let theme = PickList::new(
            &[Theme::Light, Theme::Dark, Theme::System],
            Some(self.config.ui.theme),
            Message::ThemeChanged,
        )
        .padding(10);

        Column::new()
            .push(title)
            .push(autostart)
            .push(minimize_to_tray)
            .push(language)
            .push(theme)
            .spacing(10)
            .into()
    }

    fn pomodoro_settings_view(&self) -> Element<Message> {
        let title = Text::new("番茄钟设置").size(20);

        let work_duration = TextInput::new(
            "工作时长(分钟)",
            &(self.config.pomodoro.work_duration.as_secs() / 60).to_string(),
        )
        .on_input(Message::WorkDurationChanged)
        .padding(10);

        let short_break = TextInput::new(
            "短休息时长(分钟)",
            &(self.config.pomodoro.short_break_duration.as_secs() / 60).to_string(),
        )
        .on_input(Message::ShortBreakDurationChanged)
        .padding(10);

        let long_break = TextInput::new(
            "长休息时长(分钟)",
            &(self.config.pomodoro.long_break_duration.as_secs() / 60).to_string(),
        )
        .on_input(Message::LongBreakDurationChanged)
        .padding(10);

        let long_break_interval = TextInput::new(
            "长休息间隔",
            &self.config.pomodoro.long_break_interval.to_string(),
        )
        .on_input(Message::LongBreakIntervalChanged)
        .padding(10);

        let auto_start_breaks = Checkbox::new(
            "自动开始休息",
            self.config.pomodoro.auto_start_breaks,
            Message::AutoStartBreaksChanged,
        );

        let auto_start_pomodoros = Checkbox::new(
            "自动开始下一个番茄钟",
            self.config.pomodoro.auto_start_pomodoros,
            Message::AutoStartPomodorosChanged,
        );

        let sound_enabled = Checkbox::new(
            "启用声音",
            self.config.pomodoro.sound_enabled,
            Message::SoundEnabledChanged,
        );

        let sound_volume = TextInput::new(
            "音量(0-100)",
            &self.config.pomodoro.sound_volume.to_string(),
        )
        .on_input(Message::SoundVolumeChanged)
        .padding(10);

        Column::new()
            .push(title)
            .push(work_duration)
            .push(short_break)
            .push(long_break)
            .push(long_break_interval)
            .push(auto_start_breaks)
            .push(auto_start_pomodoros)
            .push(sound_enabled)
            .push(sound_volume)
            .spacing(10)
            .into()
    }

    fn backup_settings_view(&self) -> Element<Message> {
        let title = Text::new("备份设置").size(20);

        let backup_enabled = Checkbox::new(
            "启用自动备份",
            self.config.storage.backup_enabled,
            Message::BackupEnabledChanged,
        );

        let backup_interval = TextInput::new(
            "备份间隔(小时)",
            &(self.config.storage.backup_interval.as_secs() / 3600).to_string(),
        )
        .on_input(Message::BackupIntervalChanged)
        .padding(10);

        let max_backup_count = TextInput::new(
            "最大备份数量",
            &self.config.storage.max_backup_count.to_string(),
        )
        .on_input(Message::MaxBackupCountChanged)
        .padding(10);

        Column::new()
            .push(title)
            .push(backup_enabled)
            .push(backup_interval)
            .push(max_backup_count)
            .spacing(10)
            .into()
    }
} 