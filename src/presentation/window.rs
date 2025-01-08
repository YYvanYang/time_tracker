use eframe::{
    egui::{self, ViewportBuilder},
    NativeOptions,
};
use crate::core::AppResult;
use crate::presentation::ui::TimeTrackerApp;

pub fn run_native(app: TimeTrackerApp) -> AppResult<()> {
    let viewport = ViewportBuilder::default()
        .with_inner_size([800.0, 600.0])
        .with_min_inner_size([400.0, 300.0])
        .with_title("Time Tracker")
        .with_decorations(true)
        .with_resizable(true);

    let options = NativeOptions {
        viewport,
        ..Default::default()
    };

    eframe::run_native(
        "Time Tracker",
        options,
        Box::new(|_cc| Box::new(app)),
    )?;

    Ok(())
}

pub struct Window {
    app: TimeTrackerApp,
}

impl Window {
    pub fn new(app: TimeTrackerApp) -> Self {
        Self { app }
    }

    pub fn handle_hotkey(&mut self, key: &str) {
        match key {
            "settings" => {
                self.app.set_view(crate::presentation::ui::View::Settings);
            }
            "overview" => {
                self.app.set_view(crate::presentation::ui::View::Overview);
            }
            "app_usage" => {
                self.app.set_view(crate::presentation::ui::View::AppUsage);
            }
            "pomodoro" => {
                self.app.set_view(crate::presentation::ui::View::Pomodoro);
            }
            "projects" => {
                self.app.set_view(crate::presentation::ui::View::Projects);
            }
            "statistics" => {
                self.app.set_view(crate::presentation::ui::View::Statistics);
            }
            _ => {}
        }
    }
} 