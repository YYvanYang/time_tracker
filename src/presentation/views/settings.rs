use crate::infrastructure::config::Config;
use iced::{
    widget::{Button, Checkbox, Column, Container, Row, Text, TextInput, PickList, Slider},
    Element, Length, Theme,
};
use crate::presentation::ui::styles;

#[derive(Debug, Clone)]
pub enum Message {
    ThemeChanged(Theme),
    LanguageChanged(String),
    AutoStartChanged(bool),
    MinimizeToTrayChanged(bool),
    WorkDurationChanged(String),
    ShortBreakDurationChanged(String),
    LongBreakDurationChanged(String),
    LongBreakIntervalChanged(String),
    SoundEnabledChanged(bool),
    SoundVolumeChanged(i32),
    SaveSettings,
}

pub struct SettingsView {
    config: Config,
}

impl SettingsView {
    pub fn new(config: Config) -> Self {
        Self { config }
    }

    pub fn view(&self) -> Element<Message> {
        let content = Column::new()
            .spacing(20)
            .push(self.theme_section())
            .push(self.general_section())
            .push(self.pomodoro_section())
            .push(self.sound_section())
            .push(
                Button::new(Text::new("保存设置"))
                    .on_press(Message::SaveSettings)
                    .style(styles::button::primary()),
            );

        Container::new(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .padding(20)
            .into()
    }

    fn theme_section(&self) -> Element<Message> {
        let themes = vec![Theme::Light, Theme::Dark];
        let theme_picker = PickList::new(
            "主题",
            themes.as_slice(),
            Some(Theme::Light),
            Message::ThemeChanged,
        );

        let languages = vec!["zh-CN", "en-US"];
        let language_picker = PickList::new(
            "语言",
            languages.as_slice(),
            Some("zh-CN"),
            Message::LanguageChanged,
        );

        Column::new()
            .spacing(10)
            .push(Text::new("外观设置").size(24))
            .push(theme_picker)
            .push(language_picker)
            .into()
    }

    fn general_section(&self) -> Element<Message> {
        Column::new()
            .spacing(10)
            .push(Text::new("常规设置").size(24))
            .push(
                Checkbox::new("开机自启动", true)
                    .on_toggle(Message::AutoStartChanged),
            )
            .push(
                Checkbox::new("最小化到托盘", true)
                    .on_toggle(Message::MinimizeToTrayChanged),
            )
            .into()
    }

    fn pomodoro_section(&self) -> Element<Message> {
        Column::new()
            .spacing(10)
            .push(Text::new("番茄钟设置").size(24))
            .push(
                TextInput::new(
                    "工作时长（分钟）",
                    "25",
                    Message::WorkDurationChanged,
                ),
            )
            .push(
                TextInput::new(
                    "短休息时长（分钟）",
                    "5",
                    Message::ShortBreakDurationChanged,
                ),
            )
            .push(
                TextInput::new(
                    "长休息时长（分钟）",
                    "15",
                    Message::LongBreakDurationChanged,
                ),
            )
            .push(
                TextInput::new(
                    "长休息间隔",
                    "4",
                    Message::LongBreakIntervalChanged,
                ),
            )
            .into()
    }

    fn sound_section(&self) -> Element<Message> {
        Column::new()
            .spacing(10)
            .push(Text::new("声音设置").size(24))
            .push(
                Checkbox::new("启用声音", true)
                    .on_toggle(Message::SoundEnabledChanged),
            )
            .push(
                Slider::new(0..=100, 70, Message::SoundVolumeChanged)
                    .step(1),
            )
            .into()
    }
} 