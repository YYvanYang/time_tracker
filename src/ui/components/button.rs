use eframe::egui;
use crate::ui::styles;

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
        let Button { text, style, enabled, icon: _ } = self;

        let mut response = ui.add_enabled(
            enabled,
            egui::Button::new(text)
                .fill(style.background)
                .stroke(style.border.unwrap_or_default())
                .rounding(styles::BORDER_RADIUS)
        );

        if enabled {
            if response.hovered() {
                response = response.on_hover_text("Hover");
            }
            if response.clicked() {
                response = response.on_hover_text("Clicked");
            }
        }

        response
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
} 