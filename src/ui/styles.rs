// src/ui/styles.rs

use eframe::egui::{self, Color32, RichText, Stroke, Style, TextStyle};

// 颜色定义
pub const COLOR_PRIMARY: Color32 = Color32::from_rgb(25, 118, 210);
pub const COLOR_SECONDARY: Color32 = Color32::from_rgb(156, 39, 176);
pub const COLOR_SUCCESS: Color32 = Color32::from_rgb(67, 160, 71);
pub const COLOR_WARNING: Color32 = Color32::from_rgb(251, 140, 0);
pub const COLOR_ERROR: Color32 = Color32::from_rgb(211, 47, 47);
pub const COLOR_INFO: Color32 = Color32::from_rgb(2, 136, 209);

// 尺寸定义
pub const SPACING_SMALL: f32 = 4.0;
pub const SPACING_MEDIUM: f32 = 8.0;
pub const SPACING_LARGE: f32 = 16.0;

pub const PADDING_SMALL: f32 = 4.0;
pub const PADDING_MEDIUM: f32 = 8.0;
pub const PADDING_LARGE: f32 = 16.0;

pub const BORDER_RADIUS: f32 = 4.0;
pub const ICON_SIZE: f32 = 20.0;

// 文本样式
pub fn heading() -> TextStyle {
    TextStyle::Heading
}

pub fn body() -> TextStyle {
    TextStyle::Body
}

pub fn small() -> TextStyle {
    TextStyle::Small
}

pub fn monospace() -> TextStyle {
    TextStyle::Monospace
}

// 按钮样式
pub struct ButtonStyle {
    pub background: Color32,
    pub foreground: Color32,
    pub hover_background: Color32,
    pub hover_foreground: Color32,
    pub active_background: Color32,
    pub active_foreground: Color32,
    pub border: Option<Stroke>,
}

impl ButtonStyle {
    pub fn primary() -> Self {
        Self {
            background: COLOR_PRIMARY,
            foreground: Color32::WHITE,
            hover_background: Color32::from_rgb(21, 101, 192),
            hover_foreground: Color32::WHITE,
            active_background: Color32::from_rgb(18, 84, 160),
            active_foreground: Color32::WHITE,
            border: None,
        }
    }

    pub fn secondary() -> Self {
        Self {
            background: COLOR_SECONDARY,
            foreground: Color32::WHITE,
            hover_background: Color32::from_rgb(123, 31, 162),
            hover_foreground: Color32::WHITE,
            active_background: Color32::from_rgb(106, 27, 154),
            active_foreground: Color32::WHITE,
            border: None,
        }
    }

    pub fn outlined() -> Self {
        Self {
            background: Color32::TRANSPARENT,
            foreground: COLOR_PRIMARY,
            hover_background: Color32::from_rgb(25, 118, 210).linear_multiply(0.1),
            hover_foreground: COLOR_PRIMARY,
            active_background: Color32::from_rgb(25, 118, 210).linear_multiply(0.2),
            active_foreground: COLOR_PRIMARY,
            border: Some(Stroke::new(1.0, COLOR_PRIMARY)),
        }
    }

    pub fn danger() -> Self {
        Self {
            background: COLOR_ERROR,
            foreground: Color32::WHITE,
            hover_background: Color32::from_rgb(198, 40, 40),
            hover_foreground: Color32::WHITE,
            active_background: Color32::from_rgb(183, 28, 28),
            active_foreground: Color32::WHITE,
            border: None,
        }
    }
}

// 卡片样式
pub struct CardStyle {
    pub background: Color32,
    pub border: Option<Stroke>,
    pub padding: f32,
}

impl CardStyle {
    pub fn default() -> Self {
        Self {
            background: Color32::from_gray(250),
            border: Some(Stroke::new(1.0, Color32::from_gray(230))),
            padding: PADDING_MEDIUM,
        }
    }

    pub fn elevated() -> Self {
        Self {
            background: Color32::WHITE,
            border: None,
            padding: PADDING_LARGE,
        }
    }
}

// 徽章样式
pub struct BadgeStyle {
    pub background: Color32,
    pub foreground: Color32,
}

impl BadgeStyle {
    pub fn success() -> Self {
        Self {
            background: COLOR_SUCCESS,
            foreground: Color32::WHITE,
        }
    }

    pub fn warning() -> Self {
        Self {
            background: COLOR_WARNING,
            foreground: Color32::BLACK,
        }
    }

    pub fn error() -> Self {
        Self {
            background: COLOR_ERROR,
            foreground: Color32::WHITE,
        }
    }

    pub fn info() -> Self {
        Self {
            background: COLOR_INFO,
            foreground: Color32::WHITE,
        }
    }
}

// 文本格式化
pub fn format_text(text: &str, style: TextStyle, color: Option<Color32>) -> RichText {
    let mut rich_text = RichText::new(text).text_style(style);
    if let Some(color) = color {
        rich_text = rich_text.color(color);
    }
    rich_text
}

// 布局辅助函数
pub fn centered_row<R>(ui: &mut egui::Ui, add_contents: impl FnOnce(&mut egui::Ui) -> R) -> R {
    ui.with_layout(
        egui::Layout::left_to_right(egui::Align::Center)
            .with_cross_align(egui::Align::Center),
        add_contents,
    )
}

pub fn centered_column<R>(ui: &mut egui::Ui, add_contents: impl FnOnce(&mut egui::Ui) -> R) -> R {
    ui.with_layout(
        egui::Layout::top_down(egui::Align::Center)
            .with_cross_align(egui::Align::Center),
        add_contents,
    )
}

// 绘制辅助函数
pub fn draw_card(ui: &mut egui::Ui, style: &CardStyle, add_contents: impl FnOnce(&mut egui::Ui)) {
    let mut frame = egui::Frame::none()
        .fill(style.background)
        .inner_margin(style.padding)
        .rounding(BORDER_RADIUS);

    if let Some(stroke) = style.border {
        frame = frame.stroke(stroke);
    }

    frame.show(ui, add_contents);
}

pub fn draw_badge(ui: &mut egui::Ui, text: &str, style: &BadgeStyle) {
    let frame = egui::Frame::none()
        .fill(style.background)
        .inner_margin(egui::vec2(PADDING_SMALL, PADDING_SMALL / 2.0))
        .rounding(BORDER_RADIUS);

    frame.show(ui, |ui| {
        ui.label(format_text(text, small(), Some(style.foreground)));
    });
}

pub fn draw_divider(ui: &mut egui::Ui) {
    ui.add(egui::Separator::default().spacing(SPACING_MEDIUM));
}

// 响应式布局辅助函数
pub fn available_width(ui: &egui::Ui) -> f32 {
    ui.available_width()
}

pub fn is_mobile_width(width: f32) -> bool {
    width < 600.0
}

pub fn responsive_padding(width: f32) -> f32 {
    if is_mobile_width(width) {
        PADDING_SMALL
    } else {
        PADDING_MEDIUM
    }
}

// 动画辅助函数
pub fn lerp(start: f32, end: f32, t: f32) -> f32 {
    start + (end - start) * t
}

pub fn smooth_step(t: f32) -> f32 {
    let t = t.clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_button_styles() {
        let primary = ButtonStyle::primary();
        assert_eq!(primary.background, COLOR_PRIMARY);
        assert_eq!(primary.foreground, Color32::WHITE);
        assert!(primary.border.is_none());

        let outlined = ButtonStyle::outlined();
        assert_eq!(outlined.background, Color32::TRANSPARENT);
        assert!(outlined.border.is_some());
    }

    #[test]
    fn test_badge_styles() {
        let success = BadgeStyle::success();
        assert_eq!(success.background, COLOR_SUCCESS);
        assert_eq!(success.foreground, Color32::WHITE);

        let warning = BadgeStyle::warning();
        assert_eq!(warning.background, COLOR_WARNING);
        assert_eq!(warning.foreground, Color32::BLACK);
    }

    #[test]
    fn test_responsive_layout() {
        assert!(is_mobile_width(500.0));
        assert!(!is_mobile_width(800.0));

        assert_eq!(responsive_padding(500.0), PADDING_SMALL);
        assert_eq!(responsive_padding(800.0), PADDING_MEDIUM);
    }

    #[test]
    fn test_animations() {
        assert_eq!(lerp(0.0, 10.0, 0.5), 5.0);
        assert_eq!(smooth_step(0.5), 0.5);
        assert_eq!(smooth_step(2.0), 1.0);  // 超出范围应该被限制
    }
}