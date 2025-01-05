use eframe::egui;
use crate::ui::styles;

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
        &self,
        ui: &mut egui::Ui,
        add_contents: impl FnOnce(&mut egui::Ui) -> R,
    ) -> egui::Response {
        if self.collapsible {
            if let Some(title) = &self.title {
                egui::CollapsingHeader::new(title)
                    .default_open(true)
                    .show(ui, |ui| {
                        self.draw_card_contents(ui, &self.style, add_contents);
                    })
                    .header_response
            } else {
                self.draw_frame(ui, &self.style, add_contents)
            }
        } else {
            self.draw_frame(ui, &self.style, add_contents)
        }
    }

    fn draw_frame<R>(
        &self,
        ui: &mut egui::Ui,
        style: &styles::CardStyle,
        add_contents: impl FnOnce(&mut egui::Ui) -> R,
    ) -> egui::Response {
        egui::Frame::none()
            .inner_margin(style.padding)
            .rounding(style.rounding)
            .fill(style.background)
            .stroke(style.border)
            .show(ui, |ui| {
                self.draw_card_contents(ui, style, add_contents);
            })
            .response
    }

    fn draw_card_contents<R>(
        &self,
        ui: &mut egui::Ui,
        style: &styles::CardStyle,
        add_contents: impl FnOnce(&mut egui::Ui) -> R,
    ) {
        ui.spacing_mut().item_spacing = style.spacing;
        add_contents(ui);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use eframe::egui::{Context, RawInput};

    #[test]
    fn test_card() {
        let ctx = Context::default();
        ctx.run(RawInput::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                let card = Card::new()
                    .collapsible("Test Card");
                card.show(ui, |ui| {
                    ui.label("Test content");
                });
            });
        });
    }
} 