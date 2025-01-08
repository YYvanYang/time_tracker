use iced::{
    widget::{container, Column},
    Element, Length, Theme,
};
use crate::presentation::ui::styles;
use crate::presentation::ui::Message;

pub struct Card<'a> {
    content: Column<'a, Message>,
}

impl<'a> Card<'a> {
    pub fn new() -> Self {
        Self {
            content: Column::new(),
        }
    }

    pub fn push<E>(mut self, element: E) -> Self
    where
        E: Into<Element<'a, Message>>,
    {
        self.content = self.content.push(element);
        self
    }

    pub fn spacing(mut self, spacing: f32) -> Self {
        self.content = self.content.spacing(spacing);
        self
    }

    pub fn into_element(self) -> Element<'a, Message> {
        container(self.content)
            .width(Length::Fill)
            .padding(20)
            .style(styles::container::content())
            .into()
    }
} 