use crate::error::Result;
use crate::ui::TimeTrackerApp;
use crate::config::Config;
use crate::storage::Storage;
use crate::storage::app_state::AppState;
use eframe::egui;

pub struct DialogContext<'a> {
    pub app: &'a mut TimeTrackerApp,
    pub config: &'a Config,
    pub state: &'a AppState,
    pub storage: &'a Storage,
}

impl<'a> DialogContext<'a> {
    pub fn show_error(&mut self, error: String) {
        self.app.show_error(error);
    }

    pub fn pop_dialog(&mut self) {
        self.app.pop_dialog();
    }
}

pub trait Dialog {
    fn show(&mut self, ctx: &egui::Context, dialog_ctx: &mut DialogContext) -> bool;
    fn validate(&self) -> Result<()> {
        Ok(())
    }
}

pub trait DialogHandler: std::any::Any + Send {
    fn show(&mut self, ctx: &egui::Context, dialog_ctx: &mut DialogContext) -> bool;
} 