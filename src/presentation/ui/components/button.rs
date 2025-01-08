use iced::{
    widget::button,
    Background, Color, Theme, Vector,
};
use crate::presentation::ui::styles;

pub struct Button {
    label: String,
    style: Theme,
}

impl Button {
    pub fn new(label: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            style: styles::button::primary(),
        }
    }

    pub fn style(mut self, style: Theme) -> Self {
        self.style = style;
        self
    }
} 