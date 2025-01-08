use iced::{
    widget::text,
    Color,
};

pub fn primary() -> text::Appearance {
    text::Appearance {
        color: Some(Color::from_rgb(0.2, 0.5, 0.8)),
    }
}

pub fn success() -> text::Appearance {
    text::Appearance {
        color: Some(Color::from_rgb(0.2, 0.8, 0.2)),
    }
}

pub fn info() -> text::Appearance {
    text::Appearance {
        color: Some(Color::from_rgb(0.2, 0.7, 0.9)),
    }
}

pub fn warning() -> text::Appearance {
    text::Appearance {
        color: Some(Color::from_rgb(0.9, 0.6, 0.1)),
    }
}

pub fn secondary() -> text::Appearance {
    text::Appearance {
        color: Some(Color::from_rgb(0.6, 0.6, 0.6)),
    }
} 