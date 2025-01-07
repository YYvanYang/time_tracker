use super::base::{Dialog, DialogContext};
use crate::error::Result;
use crate::ui::TimeTrackerApp;
use eframe::egui;

pub struct ConfirmationDialog {
    pub title: String,
    pub message: String,
    pub on_confirm: Option<Box<dyn FnOnce(&mut TimeTrackerApp) -> Result<()> + Send>>,
    pub on_cancel: Option<Box<dyn FnOnce(&mut TimeTrackerApp) -> Result<()> + Send>>,
}

impl Dialog for ConfirmationDialog {
    fn show(&mut self, ctx: &egui::Context, dialog_ctx: &mut DialogContext) -> bool {
        let mut is_open = true;
        
        egui::Window::new(&self.title)
            .collapsible(false)
            .resizable(false)
            .open(&mut is_open)
            .show(ctx, |ui| {
                ui.label(&self.message);
                ui.horizontal(|ui| {
                    if ui.button("确认").clicked() {
                        if let Some(on_confirm) = self.on_confirm.take() {
                            if let Err(e) = on_confirm(dialog_ctx.app) {
                                dialog_ctx.show_error(e.to_string());
                            }
                        }
                        dialog_ctx.pop_dialog();
                    }
                    if ui.button("取消").clicked() {
                        if let Some(on_cancel) = self.on_cancel.take() {
                            if let Err(e) = on_cancel(dialog_ctx.app) {
                                dialog_ctx.show_error(e.to_string());
                            }
                        }
                        dialog_ctx.pop_dialog();
                    }
                });
            });

        is_open
    }
} 