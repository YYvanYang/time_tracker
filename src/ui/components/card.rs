use eframe::egui;
use super::styles;

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
        let Card { style: _, collapsible, title } = self;

        if collapsible {
            if let Some(title) = title {
                egui::CollapsingHeader::new(title)
                    .default_open(true)
                    .show(ui, add_contents)
                    .body_returned
            } else {
                None
            }
        } else {
            Some(add_contents(ui))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use eframe::egui::Context;

    #[test]
    fn test_card() {
        let ctx = Context::default();
        ctx.run(|ctx| {
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