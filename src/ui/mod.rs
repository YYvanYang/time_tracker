pub mod dialogs;
pub mod components;
pub mod views;
pub mod styles;
pub mod theme;

pub use dialogs::*;

use crate::config::Theme as AppTheme;
use crate::error::Result;
use crate::config::Config;
use crate::storage::Storage;
use crate::app_tracker::AppTracker;
use crate::pomodoro::PomodoroTimer;
use crate::storage::app_state::AppStateManager;
use crate::storage::app_state::Task;
use crate::tray::{TrayManager, TrayEvent};
use crate::hotkeys::HotkeyManager;
use eframe::egui;
use std::sync::{Arc, Mutex, mpsc::Receiver};
use views::*;
use chrono::Duration;
use crate::storage::{Project, Tag};
use crate::storage::app_state::AppState;
use crate::ui::dialogs::DialogHandler;

pub struct AppUsageData {
    pub name: String,
    pub duration: Duration,
}

pub struct TimeTrackerApp {
    config: Arc<Mutex<Config>>,
    storage: Arc<Mutex<Storage>>,
    pomodoro_timer: Arc<Mutex<PomodoroTimer>>,
    app_tracker: Arc<Mutex<AppTracker>>,
    app_state_manager: Arc<Mutex<AppStateManager>>,
    tray_manager: Arc<Mutex<TrayManager>>,
    hotkey_manager: Arc<Mutex<HotkeyManager>>,
    tray_event_receiver: Receiver<TrayEvent>,
    current_project: Option<Project>,
    ui_state: UiState,
    pub selected_time_range: usize,
    pub usage_data: Vec<AppUsageData>,
    pub tasks: Vec<Task>,
    pub projects: Vec<Project>,
    show_add_project_dialog: bool,
}

impl TimeTrackerApp {
    pub fn show_confirmation_dialog(
        &mut self,
        title: String,
        message: String,
        on_confirm: Box<dyn FnOnce(&mut TimeTrackerApp) -> Result<()> + Send>,
    ) {
        use crate::ui::dialogs::ConfirmationDialog;
        self.push_dialog(Box::new(ConfirmationDialog {
            title,
            message,
            on_confirm: Some(on_confirm),
            on_cancel: None,
        }));
    }
}

[... rest of the file remains exactly the same ...]