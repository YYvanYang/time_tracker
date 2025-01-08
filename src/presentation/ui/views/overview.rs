use iced::{
    widget::{Column, Container, Row, Text},
    Element, Length,
};
use crate::presentation::ui::Message;

pub fn view<'a>() -> Element<'a, Message> {
    let content = Column::new()
        .spacing(20)
        .push(
            Text::new("Overview")
                .size(24)
        )
        .push(
            Row::new()
                .spacing(20)
                .push(
                    Container::new(
                        Column::new()
                            .spacing(10)
                            .push(Text::new("Current Activity"))
                            .push(Text::new("No activity"))
                    )
                    .width(Length::Fill)
                )
                .push(
                    Container::new(
                        Column::new()
                            .spacing(10)
                            .push(Text::new("Today's Stats"))
                            .push(Text::new("No data"))
                    )
                    .width(Length::Fill)
                )
        );

    Container::new(content)
        .width(Length::Fill)
        .height(Length::Fill)
        .padding(20)
        .into()
}