use super::base::{Dialog, DialogContext};
use eframe::egui;

pub struct AboutDialog {
    pub version: String,
    pub author: String,
    pub description: String,
}

impl AboutDialog {
    pub fn new() -> Self {
        Self {
            version: env!("CARGO_PKG_VERSION").to_string(),
            author: "Your Name".to_string(),
            description: "一个简单的时间跟踪工具".to_string(),
        }
    }
}

impl Dialog for AboutDialog {
    fn show(&mut self, ctx: &egui::Context, dialog_ctx: &mut DialogContext) -> bool {
        let mut is_open = true;
        
        egui::Window::new("关于")
            .collapsible(false)
            .resizable(false)
            .open(&mut is_open)
            .show(ctx, |ui| {
                ui.vertical_centered(|ui| {
                    ui.heading("Time Tracker");
                    ui.label(format!("版本: {}", self.version));
                    ui.label(format!("作者: {}", self.author));
                    ui.label(&self.description);
                    ui.add_space(8.0);
                    if ui.button("确定").clicked() {
                        dialog_ctx.pop_dialog();
                    }
                });
            });

        is_open
    }
} 