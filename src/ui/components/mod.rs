// src/ui/components/mod.rs

mod button;
mod card;
mod chart;
mod dialog;
mod input;
mod progress;

pub use button::*;
pub use card::*;
pub use chart::*;
pub use dialog::*;
pub use input::*;
pub use progress::*;

use crate::error::Result;
use eframe::egui;
use super::styles;

// 按钮组件
#[derive(Debug)]
pub struct Button {
    text: String,
    style: styles::ButtonStyle,
    enabled: bool,
    icon: Option<&'static str>,
}

impl Button {
    pub fn new(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            style: styles::ButtonStyle::primary(),
            enabled: true,
            icon: None,
        }
    }

    pub fn with_style(mut self, style: styles::ButtonStyle) -> Self {
        self.style = style;
        self
    }

    pub fn with_icon(mut self, icon: &'static str) -> Self {
        self.icon = Some(icon);
        self
    }

    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }

    pub fn show(self, ui: &mut egui::Ui) -> egui::Response {
        let Button {
            text,
            style,
            enabled,
            icon,
        } = self;

        let mut response = ui.add_enabled(
            enabled,
            egui::Button::new(
                styles::format_text(&text, styles::body(), Some(style.foreground))
            )
            .fill(style.background)
            .stroke(style.border.unwrap_or_default())
            .rounding(styles::BORDER_RADIUS)
        );

        if enabled {
            if response.hovered() {
                response.widget_info(|info| {
                    info.fill = Some(style.hover_background);
                    info.text_color = Some(style.hover_foreground);
                });
            }

            if response.clicked() {
                response.widget_info(|info| {
                    info.fill = Some(style.active_background);
                    info.text_color = Some(style.active_foreground);
                });
            }
        }

        response
    }
}

// 卡片组件
pub struct Card {
    style: styles::CardStyle,
    collapsible: bool,
    title: Option<String>,
}

impl Card {
    pub fn new() -> Self {
        Self {
            style: styles::CardStyle::default(),
            collapsible: false,
            title: None,
        }
    }

    pub fn with_style(mut self, style: styles::CardStyle) -> Self {
        self.style = style;
        self
    }

    pub fn collapsible(mut self, title: impl Into<String>) -> Self {
        self.collapsible = true;
        self.title = Some(title.into());
        self
    }

    pub fn show<R>(
        self,
        ui: &mut egui::Ui,
        add_contents: impl FnOnce(&mut egui::Ui) -> R,
    ) -> Option<R> {
        let Card {
            style,
            collapsible,
            title,
        } = self;

        if collapsible {
            if let Some(title) = title {
                egui::CollapsingHeader::new(title)
                    .default_open(true)
                    .show(ui, |ui| {
                        styles::draw_card(ui, &style, add_contents)
                    })
                    .body_returned
            } else {
                None
            }
        } else {
            Some(styles::draw_card(ui, &style, add_contents))
        }
    }
}

// 图表组件
pub struct Chart {
    data: Vec<(f64, f64)>,
    bounds: (f64, f64, f64, f64),
    width: f32,
    height: f32,
    show_points: bool,
    show_lines: bool,
    color: Color32,
}

impl Chart {
    pub fn new(data: Vec<(f64, f64)>) -> Self {
        let (min_x, max_x, min_y, max_y) = if data.is_empty() {
            (0.0, 1.0, 0.0, 1.0)
        } else {
            let min_x = data.iter().map(|(x, _)| *x).fold(f64::INFINITY, f64::min);
            let max_x = data.iter().map(|(x, _)| *x).fold(f64::NEG_INFINITY, f64::max);
            let min_y = data.iter().map(|(_, y)| *y).fold(f64::INFINITY, f64::min);
            let max_y = data.iter().map(|(_, y)| *y).fold(f64::NEG_INFINITY, f64::max);
            (min_x, max_x, min_y, max_y)
        };

        Self {
            data,
            bounds: (min_x, max_x, min_y, max_y),
            width: 300.0,
            height: 200.0,
            show_points: true,
            show_lines: true,
            color: styles::COLOR_PRIMARY,
        }
    }

    pub fn with_size(mut self, width: f32, height: f32) -> Self {
        self.width = width;
        self.height = height;
        self
    }

    pub fn with_bounds(mut self, min_x: f64, max_x: f64, min_y: f64, max_y: f64) -> Self {
        self.bounds = (min_x, max_x, min_y, max_y);
        self
    }

    pub fn show_points(mut self, show: bool) -> Self {
        self.show_points = show;
        self
    }

    pub fn show_lines(mut self, show: bool) -> Self {
        self.show_lines = show;
        self
    }

    pub fn with_color(mut self, color: Color32) -> Self {
        self.color = color;
        self
    }

    pub fn show(&self, ui: &mut egui::Ui) {
        let Chart {
            data,
            bounds,
            width,
            height,
            show_points,
            show_lines,
            color,
        } = self;

        let (rect, response) = ui.allocate_exact_size(
            egui::vec2(*width, *height),
            egui::Sense::hover()
        );

        if !ui.is_rect_visible(rect) {
            return;
        }

        let painter = ui.painter_at(rect);
        
        let to_screen = egui::emath::RectTransform::from_to(
            egui::Rect::from_min_max(
                egui::pos2(bounds.0 as f32, bounds.2 as f32),
                egui::pos2(bounds.1 as f32, bounds.3 as f32),
            ),
            rect,
        );

        let mut shapes = Vec::new();
        
        let grid_color = color.linear_multiply(0.2);
        for i in 0..=10 {
            let x = bounds.0 + (bounds.1 - bounds.0) * (i as f64 / 10.0);
            let start = to_screen * egui::pos2(x as f32, bounds.2 as f32);
            let end = to_screen * egui::pos2(x as f32, bounds.3 as f32);
            shapes.push(egui::Shape::line_segment(
                [start, end],
                (1.0, grid_color),
            ));

            let y = bounds.2 + (bounds.3 - bounds.2) * (i as f64 / 10.0);
            let start = to_screen * egui::pos2(bounds.0 as f32, y as f32);
            let end = to_screen * egui::pos2(bounds.1 as f32, y as f32);
            shapes.push(egui::Shape::line_segment(
                [start, end],
                (1.0, grid_color),
            ));
        }

        if show_lines && data.len() >= 2 {
            for points in data.windows(2) {
                let start = to_screen * egui::pos2(points[0].0 as f32, points[0].1 as f32);
                let end = to_screen * egui::pos2(points[1].0 as f32, points[1].1 as f32);
                shapes.push(egui::Shape::line_segment(
                    [start, end],
                    (2.0, color),
                ));
            }
        }

        if show_points {
            for (x, y) in &data {
                let center = to_screen * egui::pos2(*x as f32, *y as f32);
                shapes.push(egui::Shape::circle_filled(
                    center,
                    4.0,
                    color,
                ));
            }
        }

        painter.extend(shapes);
    }
}

// 进度条组件
pub struct ProgressBar {
    progress: f32,
    show_percentage: bool,
    color: Color32,
    height: f32,
}

impl ProgressBar {
    pub fn new(progress: f32) -> Self {
        Self {
            progress: progress.clamp(0.0, 1.0),
            show_percentage: true,
            color: styles::COLOR_PRIMARY,
            height: 8.0,
        }
    }

    pub fn show_percentage(mut self, show: bool) -> Self {
        self.show_percentage = show;
        self
    }

    pub fn with_color(mut self, color: Color32) -> Self {
        self.color = color;
        self
    }

    pub fn with_height(mut self, height: f32) -> Self {
        self.height = height;
        self
    }

    pub fn show(self, ui: &mut egui::Ui) -> egui::Response {
        let ProgressBar {
            progress,
            show_percentage,
            color,
            height,
        } = self;

        let desired_size = egui::vec2(ui.available_width(), height);
        let (rect, response) = ui.allocate_at_least(desired_size, egui::Sense::hover());

        if ui.is_rect_visible(rect) {
            let painter = ui.painter();

            // 绘制背景
            painter.rect_filled(
                rect,
                styles::BORDER_RADIUS,
                color.linear_multiply(0.2),
            );

            // 绘制进度条
            let progress_width = rect.width() * progress;
            if progress_width > 0.0 {
                painter.rect_filled(
                    egui::Rect::from_min_max(
                        rect.min,
                        egui::pos2(rect.min.x + progress_width, rect.max.y),
                    ),
                    styles::BORDER_RADIUS,
                    color,
                );
            }

            // 显示百分比
            if show_percentage {
                let text = format!("{}%", (progress * 100.0).round() as i32);
                let galley = ui.painter().layout_no_wrap(
                    text,
                    styles::small().into(),
                    color,
                );
                let text_pos = egui::pos2(
                    rect.center().x - galley.size().x / 2.0,
                    rect.center().y - galley.size().y / 2.0,
                );
                painter.galley(text_pos, galley);
            }
        }

        response
    }
}

// 标签组件
pub struct Tag {
    text: String,
    color: Color32,
    removable: bool,
}

impl Tag {
    pub fn new(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            color: styles::COLOR_PRIMARY,
            removable: false,
        }
    }

    pub fn with_color(mut self, color: Color32) -> Self {
        self.color = color;
        self
    }

    pub fn removable(mut self) -> Self {
        self.removable = true;
        self
    }

    pub fn show(self, ui: &mut egui::Ui) -> Option<()> {
        let Tag {
            text,
            color,
            removable,
        } = self;

        let margin = egui::vec2(6.0, 2.0);
        let total_extra = margin * 2.0;

        let text = styles::format_text(&text, styles::small(), Some(color));
        let galley = text.into_galley(ui, Some(false), 0.0, styles::small().into());
        let mut desired_size = galley.size() + total_extra;
        if removable {
            desired_size.x += 16.0; // 为删除按钮留出空间
        }

        let (rect, mut response) = ui.allocate_exact_size(desired_size, egui::Sense::click());

        if ui.is_rect_visible(rect) {
            let painter = ui.painter();

            // 绘制背景
            painter.rect_filled(
                rect,
                styles::BORDER_RADIUS,
                color.linear_multiply(0.2),
            );

            // 绘制文本
            let text_pos = rect.min + margin;
            painter.galley(text_pos, galley);

            // 绘制删除按钮
            if removable {
                let button_rect = egui::Rect::from_min_size(
                    egui::pos2(rect.max.x - 16.0 - margin.x, rect.min.y + margin.y),
                    egui::vec2(16.0, 16.0),
                );
                if response.clicked_by(egui::PointerButton::Primary)
                    && button_rect.contains(response.hover_pos().unwrap_or_default())
                {
                    return None;
                }

                painter.circle_stroke(
                    button_rect.center(),
                    6.0,
                    (1.0, color),
                );
                painter.line_segment(
                    [
                        button_rect.center() + egui::vec2(-3.0, -3.0),
                        button_rect.center() + egui::vec2(3.0, 3.0),
                    ],
                    (1.0, color),
                );
                painter.line_segment(
                    [
                        button_rect.center() + egui::vec2(3.0, -3.0),
                        button_rect.center() + egui::vec2(-3.0, 3.0),
                    ],
                    (1.0, color),
                );
            }
        }

        Some(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use eframe::egui::Context;

    #[test]
    fn test_button() {
        let ctx = Context::default();
        ctx.run(|ctx| {
            let mut used_size = egui::Vec2::ZERO;
            egui::CentralPanel::default().show(ctx, |ui| {
                let response = Button::new("Test")
                    .with_style(styles::ButtonStyle::primary())
                    .show(ui);
                used_size = response.rect.size();
            });
            assert!(used_size.x > 0.0 && used_size.y > 0.0);
        });
    }

    #[test]
    fn test_progress_bar() {
        let ctx = Context::default();
        ctx.run(|ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                let response = ProgressBar::new(0.5)
                    .show_percentage(true)
                    .show(ui);
                assert!(response.rect.width() > 0.0);
            });
        });
    }

    #[test]
    fn test_tag() {
        let ctx = Context::default();
        ctx.run(|ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                let result = Tag::new("Test")
                    .removable()
                    .show(ui);
                assert!(result.is_some());
            });
        });
    }
}