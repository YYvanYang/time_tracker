use eframe::egui;
use crate::ui::styles;

pub struct Chart {
    data: Vec<(f64, f64)>,
    bounds: (f64, f64, f64, f64),
    width: f32,
    height: f32,
    show_points: bool,
    show_lines: bool,
    color: egui::Color32,
    stroke_width: f32,
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
            stroke_width: 2.0,
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

    pub fn with_color(mut self, color: egui::Color32) -> Self {
        self.color = color;
        self
    }

    pub fn show(&self, ui: &mut egui::Ui) {
        let painter = ui.painter();
        let mut shapes = Vec::new();

        // 计算坐标转换
        let rect = ui.available_rect_before_wrap();
        let to_screen = egui::emath::RectTransform::from_to(
            egui::Rect::from_min_size(
                egui::pos2(self.bounds.0 as f32, self.bounds.2 as f32),
                egui::vec2(
                    (self.bounds.1 - self.bounds.0) as f32,
                    (self.bounds.3 - self.bounds.2) as f32,
                ),
            ),
            rect,
        );

        // 绘制线
        if self.show_lines && self.data.len() >= 2 {
            for points in self.data.windows(2) {
                let start = to_screen * egui::pos2(points[0].0 as f32, points[0].1 as f32);
                let end = to_screen * egui::pos2(points[1].0 as f32, points[1].1 as f32);
                shapes.push(egui::Shape::line_segment(
                    [start, end],
                    egui::Stroke::new(self.stroke_width, self.color),
                ));
            }
        }

        // 绘制点
        if self.show_points {
            for (x, y) in &self.data {
                let center = to_screen * egui::pos2(*x as f32, *y as f32);
                shapes.push(egui::Shape::circle_filled(
                    center,
                    4.0,
                    self.color,
                ));
            }
        }

        painter.extend(shapes);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use eframe::egui::{Context, RawInput};

    #[test]
    fn test_chart() {
        let ctx = Context::default();
        ctx.run(RawInput::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                let data = vec![(0.0, 0.0), (1.0, 1.0), (2.0, 4.0)];
                let chart = Chart::new(data)
                    .with_size(200.0, 100.0);
                chart.show(ui);
            });
        });
    }
} 