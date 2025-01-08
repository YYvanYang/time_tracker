use iced::{
    widget::button,
    Background, Color, Theme, Vector,
};

pub fn primary() -> button::Appearance {
    button::Appearance {
        background: Some(Background::Color(Color::from_rgb(0.2, 0.5, 0.8))),
        text_color: Color::WHITE,
        ..Default::default()
    }
}

pub fn warning() -> button::Appearance {
    button::Appearance {
        background: Some(Background::Color(Color::from_rgb(0.9, 0.6, 0.1))),
        text_color: Color::WHITE,
        ..Default::default()
    }
}

pub fn danger() -> button::Appearance {
    button::Appearance {
        background: Some(Background::Color(Color::from_rgb(0.8, 0.2, 0.2))),
        text_color: Color::WHITE,
        ..Default::default()
    }
}

pub fn active() -> Theme {
    Theme::Custom(Box::new(Active))
}

pub struct Primary;
pub struct Active;

impl button::StyleSheet for Primary {
    type Style = Theme;

    fn active(&self) -> button::Appearance {
        button::Appearance {
            background: Some(Background::Color(Color::from_rgb(0.11, 0.42, 0.87))),
            border_radius: 4.0,
            shadow_offset: Vector::new(0.0, 0.0),
            text_color: Color::WHITE,
            ..Default::default()
        }
    }

    fn hovered(&self) -> button::Appearance {
        button::Appearance {
            background: Some(Background::Color(Color::from_rgb(0.2, 0.5, 0.9))),
            ..self.active()
        }
    }

    fn pressed(&self) -> button::Appearance {
        button::Appearance {
            background: Some(Background::Color(Color::from_rgb(0.1, 0.35, 0.8))),
            ..self.active()
        }
    }

    fn disabled(&self) -> button::Appearance {
        button::Appearance {
            background: Some(Background::Color(Color::from_rgb(0.5, 0.5, 0.5))),
            ..self.active()
        }
    }
}

impl button::StyleSheet for Active {
    type Style = Theme;

    fn active(&self) -> button::Appearance {
        button::Appearance {
            background: Some(Background::Color(Color::from_rgb(0.2, 0.6, 0.9))),
            border_radius: 4.0,
            shadow_offset: Vector::new(0.0, 0.0),
            text_color: Color::WHITE,
            ..Default::default()
        }
    }

    fn hovered(&self) -> button::Appearance {
        button::Appearance {
            background: Some(Background::Color(Color::from_rgb(0.3, 0.7, 0.95))),
            ..self.active()
        }
    }

    fn pressed(&self) -> button::Appearance {
        button::Appearance {
            background: Some(Background::Color(Color::from_rgb(0.15, 0.5, 0.85))),
            ..self.active()
        }
    }

    fn disabled(&self) -> button::Appearance {
        button::Appearance {
            background: Some(Background::Color(Color::from_rgb(0.5, 0.5, 0.5))),
            ..self.active()
        }
    }
} 