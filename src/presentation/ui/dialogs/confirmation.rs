use iced::{widget::{button, text, Column, Row}, Element, Length};
use crate::presentation::ui::dialogs::base::Dialog;
use crate::presentation::ui::Message;
use crate::presentation::ui::styles::button::{ButtonStyle, PrimaryButton, DangerButton};

pub struct ConfirmationDialog {
    title: String,
    message: String,
    visible: bool,
    on_confirm: Box<dyn Fn() -> Message>,
    on_cancel: Box<dyn Fn() -> Message>,
}

impl ConfirmationDialog {
    pub fn new<F1, F2>(title: String, message: String, on_confirm: F1, on_cancel: F2) -> Self
    where
        F1: Fn() -> Message + 'static,
        F2: Fn() -> Message + 'static,
    {
        Self {
            title,
            message,
            visible: false,
            on_confirm: Box::new(on_confirm),
            on_cancel: Box::new(on_cancel),
        }
    }
}

impl Dialog for ConfirmationDialog {
    fn title(&self) -> String {
        self.title.clone()
    }

    fn view(&self) -> Element<Message> {
        let content = Column::new()
            .spacing(20)
            .width(Length::Fill)
            .push(text(&self.message))
            .push(
                Row::new()
                    .spacing(10)
                    .push(
                        button("取消")
                            .style(ButtonStyle::Secondary)
                            .on_press((self.on_cancel)()),
                    )
                    .push(
                        button("确认")
                            .style(ButtonStyle::Primary)
                            .on_press((self.on_confirm)()),
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