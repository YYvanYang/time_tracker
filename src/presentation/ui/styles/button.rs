use iced::widget::button;
use iced::Color;

#[derive(Debug, Clone, Copy)]
pub enum ButtonStyle {
    Primary,
    Secondary,
}

impl Default for ButtonStyle {
    fn default() -> Self {
        ButtonStyle::Primary
    }
}

pub struct PrimaryButton;

impl button::StyleSheet for PrimaryButton {
    type Style = ButtonStyle;

    fn active(&self, style: &Self::Style) -> button::Appearance {
        match style {
            ButtonStyle::Primary => button::Appearance {
                background: Some(Color::from_rgb(0.2, 0.5, 0.8).into()),
                border_radius: 4.0,
                border_width: 1.0,
                border_color: Color::from_rgb(0.1, 0.4, 0.7),
                text_color: Color::WHITE,
                ..Default::default()
            },
            ButtonStyle::Secondary => button::Appearance {
                background: Some(Color::from_rgb(0.8, 0.8, 0.8).into()),
                border_radius: 4.0,
                border_width: 1.0,
                border_color: Color::from_rgb(0.7, 0.7, 0.7),
                text_color: Color::BLACK,
                ..Default::default()
            },
        }
    }

    fn hovered(&self, style: &Self::Style) -> button::Appearance {
        match style {
            ButtonStyle::Primary => button::Appearance {
                background: Some(Color::from_rgb(0.3, 0.6, 0.9).into()),
                ..self.active(style)
            },
            ButtonStyle::Secondary => button::Appearance {
                background: Some(Color::from_rgb(0.9, 0.9, 0.9).into()),
                ..self.active(style)
            },
        }
    }

    fn pressed(&self, style: &Self::Style) -> button::Appearance {
        match style {
            ButtonStyle::Primary => button::Appearance {
                background: Some(Color::from_rgb(0.1, 0.4, 0.7).into()),
                ..self.active(style)
            },
            ButtonStyle::Secondary => button::Appearance {
                background: Some(Color::from_rgb(0.7, 0.7, 0.7).into()),
                ..self.active(style)
            },
        }
    }

    fn disabled(&self, style: &Self::Style) -> button::Appearance {
        match style {
            ButtonStyle::Primary => button::Appearance {
                background: Some(Color::from_rgb(0.5, 0.5, 0.5).into()),
                ..self.active(style)
            },
            ButtonStyle::Secondary => button::Appearance {
                background: Some(Color::from_rgb(0.8, 0.8, 0.8).into()),
                ..self.active(style)
            },
        }
    }
}

pub struct DangerButton;

impl button::StyleSheet for DangerButton {
    type Style = ButtonStyle;

    fn active(&self, style: &Self::Style) -> button::Appearance {
        match style {
            ButtonStyle::Primary => button::Appearance {
                background: Some(Color::from_rgb(0.8, 0.2, 0.2).into()),
                border_radius: 4.0,
                border_width: 1.0,
                border_color: Color::from_rgb(0.7, 0.1, 0.1),
                text_color: Color::WHITE,
                ..Default::default()
            },
            ButtonStyle::Secondary => button::Appearance {
                background: Some(Color::from_rgb(0.9, 0.3, 0.3).into()),
                border_radius: 4.0,
                border_width: 1.0,
                border_color: Color::from_rgb(0.8, 0.2, 0.2),
                text_color: Color::WHITE,
                ..Default::default()
            },
        }
    }

    fn hovered(&self, style: &Self::Style) -> button::Appearance {
        match style {
            ButtonStyle::Primary => button::Appearance {
                background: Some(Color::from_rgb(0.9, 0.3, 0.3).into()),
                ..self.active(style)
            },
            ButtonStyle::Secondary => button::Appearance {
                background: Some(Color::from_rgb(1.0, 0.4, 0.4).into()),
                ..self.active(style)
            },
        }
    }

    fn pressed(&self, style: &Self::Style) -> button::Appearance {
        match style {
            ButtonStyle::Primary => button::Appearance {
                background: Some(Color::from_rgb(0.7, 0.1, 0.1).into()),
                ..self.active(style)
            },
            ButtonStyle::Secondary => button::Appearance {
                background: Some(Color::from_rgb(0.8, 0.2, 0.2).into()),
                ..self.active(style)
            },
        }
    }

    fn disabled(&self, style: &Self::Style) -> button::Appearance {
        match style {
            ButtonStyle::Primary => button::Appearance {
                background: Some(Color::from_rgb(0.5, 0.5, 0.5).into()),
                ..self.active(style)
            },
            ButtonStyle::Secondary => button::Appearance {
                background: Some(Color::from_rgb(0.6, 0.6, 0.6).into()),
                ..self.active(style)
            },
        }
    }
} 