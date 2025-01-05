// src/ui/styles.rs

use eframe::egui::{self, Color32, Rounding, Stroke, Vec2};

// 间距常量
pub const BORDER_RADIUS: f32 = 6.0;
pub const PADDING: f32 = 8.0;
pub const ITEM_SPACING: f32 = 4.0;
pub const SPACING_SMALL: f32 = 4.0;
pub const SPACING_MEDIUM: f32 = 8.0;
pub const SPACING_LARGE: f32 = 16.0;

// 颜色常量
pub const COLOR_PRIMARY: Color32 = Color32::from_rgb(0, 120, 215);
pub const COLOR_SUCCESS: Color32 = Color32::from_rgb(0, 180, 0);
pub const COLOR_WARNING: Color32 = Color32::from_rgb(255, 140, 0);
pub const COLOR_ERROR: Color32 = Color32::from_rgb(200, 50, 50);
pub const COLOR_INFO: Color32 = Color32::from_rgb(100, 150, 255);
pub const COLOR_TEXT: Color32 = Color32::from_rgb(240, 240, 240);
pub const COLOR_TEXT_SECONDARY: Color32 = Color32::from_rgb(160, 160, 160);
pub const COLOR_BACKGROUND: Color32 = Color32::from_rgb(32, 32, 32);
pub const COLOR_SURFACE: Color32 = Color32::from_rgb(48, 48, 48);

// 文本样式辅助函数
pub fn heading() -> egui::TextStyle {
    egui::TextStyle::Heading
}

pub fn body() -> egui::TextStyle {
    egui::TextStyle::Body
}

pub fn small() -> egui::TextStyle {
    egui::TextStyle::Small
}

pub fn format_text(text: &str, style: egui::TextStyle, color: Option<Color32>) -> egui::RichText {
    let mut rich_text = egui::RichText::new(text).text_style(style);
    if let Some(color) = color {
        rich_text = rich_text.color(color);
    }
    rich_text
}

#[derive(Clone)]
pub struct CardStyle {
    pub background: Color32,
    pub border: Stroke,
    pub rounding: Rounding,
    pub padding: Vec2,
    pub spacing: Vec2,
}

impl Default for CardStyle {
    fn default() -> Self {
        Self {
            background: Color32::from_gray(32),
            border: Stroke::new(1.0, Color32::from_gray(60)),
            rounding: Rounding::same(BORDER_RADIUS),
            padding: Vec2::splat(PADDING),
            spacing: Vec2::splat(ITEM_SPACING),
        }
    }
}

impl CardStyle {
    pub fn elevated() -> Self {
        Self {
            background: Color32::from_gray(40),
            border: Stroke::new(1.0, Color32::from_gray(80)),
            rounding: Rounding::same(BORDER_RADIUS),
            padding: Vec2::splat(PADDING),
            spacing: Vec2::splat(ITEM_SPACING),
        }
    }
}

#[derive(Clone)]
pub struct ButtonStyle {
    pub background: Color32,
    pub border: Option<Stroke>,
    pub text_color: Color32,
}

impl ButtonStyle {
    pub fn primary() -> Self {
        Self {
            background: COLOR_PRIMARY,
            border: None,
            text_color: Color32::WHITE,
        }
    }

    pub fn outlined() -> Self {
        Self {
            background: Color32::TRANSPARENT,
            border: Some(Stroke::new(1.0, Color32::from_gray(160))),
            text_color: Color32::from_gray(200),
        }
    }

    pub fn danger() -> Self {
        Self {
            background: COLOR_ERROR,
            border: None,
            text_color: Color32::WHITE,
        }
    }
}