use iced::{
    widget::{Button, Column, Container, PickList, Row, Text, TextInput},
    Element, Length,
};
use crate::presentation::ui::{Message, TimeTrackerApp, styles, Card};
use crate::infrastructure::config::Config;

pub fn view(app: &TimeTrackerApp) -> Element<Message> {
    let mut content = Column::new().spacing(20).padding(20);
    
    // 番茄钟设置卡片
    let mut pomodoro_settings = Column::new().spacing(10);
    pomodoro_settings = pomodoro_settings.push(
        Row::new()
            .spacing(10)
            .push(Text::new("工作时长(分钟):"))
            .push(
                TextInput::new(
                    "work_duration",
                    &app.config.pomodoro_work_duration.to_string(),
                    |_| Message::UpdatePomodoroWorkDuration,
                )
                .width(Length::Fixed(60.0))
            )
    );
    pomodoro_settings = pomodoro_settings.push(
        Row::new()
            .spacing(10)
            .push(Text::new("短休息时长(分钟):"))
            .push(
                TextInput::new(
                    "short_break_duration",
                    &app.config.pomodoro_short_break_duration.to_string(),
                    |_| Message::UpdatePomodoroShortBreakDuration,
                )
                .width(Length::Fixed(60.0))
            )
    );
    pomodoro_settings = pomodoro_settings.push(
        Row::new()
            .spacing(10)
            .push(Text::new("长休息时长(分钟):"))
            .push(
                TextInput::new(
                    "long_break_duration",
                    &app.config.pomodoro_long_break_duration.to_string(),
                    |_| Message::UpdatePomodoroLongBreakDuration,
                )
                .width(Length::Fixed(60.0))
            )
    );
    
    let pomodoro_card = Card::new()
        .title("番茄钟设置")
        .content(pomodoro_settings);
    content = content.push(pomodoro_card);
    
    // 主题设置卡片
    let theme_settings = Row::new()
        .spacing(10)
        .push(Text::new("主题:"))
        .push(
            PickList::new(
                &["浅色", "深色", "跟随系统"],
                Some(app.config.theme.as_str()),
                |_| Message::UpdateTheme,
            )
            .width(Length::Fixed(120.0))
        );
    
    let theme_card = Card::new()
        .title("主题设置")
        .content(theme_settings);
    content = content.push(theme_card);
    
    // 保存按钮
    let save_button = Button::new(Text::new("保存设置"))
        .style(styles::button::primary())
        .on_press(Message::SaveSettings);
    
    content = content.push(
        Container::new(save_button)
            .width(Length::Fill)
            .center_x()
            .padding(10)
    );
    
    Container::new(content)
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
}