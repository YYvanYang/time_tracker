use iced::{
    widget::{Button, Column, Container, Row, Text, PickList},
    Element, Length,
};
use crate::core::models::ExportFormat;
use crate::presentation::ui::{Message, styles};
use super::base::{Dialog, DialogContainer};

pub struct ExportDialog {
    format: ExportFormat,
}

impl ExportDialog {
    pub fn new() -> Self {
        Self {
            format: ExportFormat::CSV,
        }
    }
}

impl Dialog for ExportDialog {
    fn view(&self) -> Element<Message> {
        let content = Column::new()
            .spacing(20)
            .push(Text::new("Export").size(24))
            .push(
                Column::new()
                    .spacing(10)
                    .push(Text::new("Format"))
                    .push(
                        PickList::new(
                            &[ExportFormat::CSV, ExportFormat::JSON, ExportFormat::Excel],
                            Some(self.format),
                            |_| Message::NoOp,
                        )
                        .padding(10)
                        .width(Length::Fill),
                    ),
            )
            .push(
                Row::new()
                    .spacing(10)
                    .push(
                        Button::new(Text::new("Cancel"))
                            .style(styles::button::primary()),
                    )
                    .push(
                        Button::new(Text::new("Export"))
                            .style(styles::button::primary()),
                    ),
            );

        DialogContainer::new()
            .push(content)
            .spacing(20)
            .into_element()
    }

    fn update(&mut self, message: Message) {
        // TODO: 实现更新逻辑
    }
} 