use iced::{
    window::{self, Position},
    Application, Settings,
};
use crate::core::AppResult;
use crate::presentation::ui::TimeTrackerApp;

pub struct Window {
    app: TimeTrackerApp,
}

impl Window {
    pub fn new(app: TimeTrackerApp) -> Self {
        Self { app }
    }

    pub fn run(self) -> AppResult<()> {
        let settings = Settings {
            window: window::Settings {
                size: (800, 600),
                position: Position::Centered,
                min_size: Some((400, 300)),
                ..Default::default()
            },
            ..Default::default()
        };

        Ok(())
    }

    pub fn show(&self) -> AppResult<()> {
        Ok(())
    }

    pub fn hide(&self) -> AppResult<()> {
        Ok(())
    }
} 