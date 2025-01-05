use eframe::egui;
use super::styles;

pub struct ProgressBar {
    progress: f32,
    show_percentage: bool,
    color: egui::Color32,
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

    pub fn with_color(mut self, color: egui::Color32) -> Self {
        self.color = color;
        self
    }

    pub fn show(&self, ui: &mut egui::Ui) -> egui::Response {
        let desired_size = egui::vec2(ui.available_width(), self.height);
        let (rect, response) = ui.allocate_at_least(desired_size, egui::Sense::hover());

        if ui.is_rect_visible(rect) {
            let painter = ui.painter();
            
            // 绘制背景
            painter.rect_filled(
                rect,
                styles::BORDER_RADIUS,
                self.color.linear_multiply(0.2),
            );

            // 绘制进度条
            let progress_width = rect.width() * self.progress;
            if progress_width > 0.0 {
                painter.rect_filled(
                    egui::Rect::from_min_max(
                        rect.min,
                        egui::pos2(rect.min.x + progress_width, rect.max.y),
                    ),
                    styles::BORDER_RADIUS,
                    self.color,
                );
            }

            // 显示百分比
            if self.show_percentage {
                let text = format!("{}%", (self.progress * 100.0).round() as i32);
                let galley = ui.painter().layout_no_wrap(
                    text,
                    styles::small().into(),
                    self.color,
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