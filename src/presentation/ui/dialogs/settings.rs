use iced::{widget::{button, text, Column, Row}, Element, Length};
use crate::presentation::ui::dialogs::base::Dialog;
use crate::presentation::ui::Message;
use crate::presentation::ui::styles::button::{ButtonStyle, PrimaryButton};
use crate::domain::config::AppConfig;

pub struct SettingsDialog {
    title: String,
    config: AppConfig,
    visible: bool,
    on_save: Box<dyn Fn(AppConfig) -> Message>,
    on_cancel: Box<dyn Fn() -> Message>,
}

impl SettingsDialog {
    pub fn new<F1, F2>(config: AppConfig, on_save: F1, on_cancel: F2) -> Self
    where
        F1: Fn(AppConfig) -> Message + 'static,
        F2: Fn() -> Message + 'static,
    {
        Self {
            title: "设置".to_string(),
            config,
            visible: false,
            on_save: Box::new(on_save),
            on_cancel: Box::new(on_cancel),
        }
    }
}

impl Dialog for SettingsDialog {
    fn title(&self) -> String {
        self.title.clone()
    }

    fn view(&self) -> Element<Message> {
        let content = Column::new()
            .spacing(20)
            .width(Length::Fill)
            .push(text("设置"))
            .push(
                Row::new()
                    .spacing(10)
                    .push(
                        button("取消")
                            .style(ButtonStyle::Secondary)
                            .on_press((self.on_cancel)()),
                    )
                    .push(
                        button("保存")
                            .style(ButtonStyle::Primary)
                            .on_press((self.on_save)(self.config.clone())),
                    ),
            );

        content.into()
    }

    fn update(&mut self, message: Message) {
        // 处理消息
    }

    fn show(&mut self) {
        self.visible = true;
    }

    fn hide(&mut self) {
        self.visible = false;
    }

    fn is_visible(&self) -> bool {
        self.visible
    }
} 