pub mod ui;
pub mod views;
pub mod state;
pub mod handlers;

mod tray;
mod window;

pub use ui::{TimeTrackerApp, View, DialogResult};
pub use tray::TrayIcon;
pub use window::MainWindow; 