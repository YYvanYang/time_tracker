use super::base::{Dialog, DialogContext};
use chrono::{DateTime, Local};
use eframe::egui;

pub struct DateRangeDialog {
    pub start_date: DateTime<Local>,
    pub end_date: DateTime<Local>,
    pub open: bool,
    pub on_close: Option<Box<dyn FnOnce(Option<(DateTime<Local>, DateTime<Local>)>)>>,
    calendar_visible: bool,
    editing_start: bool,
}

impl Default for DateRangeDialog {
    fn default() -> Self {
        Self {
            start_date: Local::now(),
            end_date: Local::now(),
            open: false,
            on_close: None,
            calendar_visible: false,
            editing_start: false,
        }
    }
}

impl Dialog for DateRangeDialog {
    fn show(&mut self, ctx: &egui::Context, _dialog_ctx: &mut DialogContext) -> bool {
        if !self.open {
            return false;
        }

        let mut is_open = true;
        egui::Window::new("选择日期范围")
            .collapsible(false)
            .resizable(false)
            .open(&mut is_open)
            .show(ctx, |ui| {
                // ... 日期选择器的具体实现
            });

        is_open
    }
} 