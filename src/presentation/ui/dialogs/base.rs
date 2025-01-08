use iced::{
    widget::{Column, Container},
    Element, Length,
};
use crate::presentation::ui::{Message, styles};

pub struct DialogContext {
    pub is_open: bool,
    pub result: Option<Message>,
}

impl DialogContext {
    pub fn new() -> Self {
        Self {
            is_open: false,
            result: None,
        }
    }
}

pub trait Dialog {
    fn view(&self) -> Element<Message>;
    fn update(&mut self, message: Message);
}

pub struct DialogContainer<'a> {
    content: Column<'a, Message>,
}

impl<'a> DialogContainer<'a> {
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
        Container::new(self.content)
            .width(Length::Fill)
            .height(Length::Fill)
            .padding(20)
            .style(styles::container::content())
            .into()
    }
} 