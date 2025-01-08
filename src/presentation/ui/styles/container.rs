use iced::{
    widget::container,
    Background, Color, Theme,
};

pub fn header() -> Theme {
    Theme::Custom(Box::new(Header))
}

pub fn content() -> Theme {
    Theme::Custom(Box::new(Content))
}

pub struct Header;
pub struct Content;

impl container::StyleSheet for Header {
    type Style = Theme;

    fn appearance(&self, _style: &Self::Style) -> container::Appearance {
        container::Appearance {
            background: Some(Background::Color(Color::from_rgb(0.95, 0.95, 0.95))),
            border_radius: 0.0,
            border_width: 0.0,
            border_color: Color::TRANSPARENT,
            ..Default::default()
        }
    }
}

impl container::StyleSheet for Content {
    type Style = Theme;

    fn appearance(&self, _style: &Self::Style) -> container::Appearance {
        container::Appearance {
            background: Some(Background::Color(Color::from_rgb(1.0, 1.0, 1.0))),
            border_radius: 0.0,
            border_width: 0.0,
            border_color: Color::TRANSPARENT,
            ..Default::default()
        }
    }
} 