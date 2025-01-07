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
use crate::storage::app_state::{Task, AppState};
use crate::tray::{TrayManager, TrayEvent};
use crate::hotkeys::HotkeyManager;
use eframe::egui;
use std::sync::{Arc, Mutex, mpsc::Receiver};
use views::*;
use chrono::Duration;
use crate::storage::{Project, Tag};
use crate::ui::dialogs::DialogHandler;

pub struct AppUsageData {
    pub name: String,
    pub duration: Duration,
}

#[derive(Debug, Clone, Default)]
pub struct DialogResult {
    pub success: bool,
    pub data: Option<serde_json::Value>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum View {
    Overview,
    AppUsage,
    Pomodoro,
    Projects,
    Statistics,
    Settings,
}

impl Default for View {
    fn default() -> Self {
        Self::Overview
    }
}

#[derive(Default)]
struct UiState {
    current_view: View,
    dialog_stack: Vec<Box<dyn DialogHandler + Send>>,
    loading: bool,
    error: Option<String>,
    current_note: Option<String>,
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
    pub fn new(
        config: Arc<Mutex<Config>>,
        storage: Arc<Mutex<Storage>>,
        pomodoro_timer: Arc<Mutex<PomodoroTimer>>,
        app_tracker: Arc<Mutex<AppTracker>>,
        app_state_manager: Arc<Mutex<AppStateManager>>,
        tray_manager: Arc<Mutex<TrayManager>>,
        hotkey_manager: Arc<Mutex<HotkeyManager>>,
        tray_event_receiver: Receiver<TrayEvent>,
    ) -> Self {
        Self {
            config,
            storage,
            pomodoro_timer,
            app_tracker,
            app_state_manager,
            tray_manager,
            hotkey_manager,
            tray_event_receiver,
            current_project: None,
            ui_state: UiState::default(),
            selected_time_range: 0,
            usage_data: Vec::new(),
            tasks: Vec::new(),
            projects: Vec::new(),
            show_add_project_dialog: false,
        }
    }

    pub fn show_error(&mut self, error: String) {
        self.ui_state.error = Some(error);
    }

    pub fn push_dialog(&mut self, dialog: Box<dyn DialogHandler + Send>) {
        self.ui_state.dialog_stack.push(dialog);
    }

    pub fn pop_dialog(&mut self) -> Option<Box<dyn DialogHandler + Send>> {
        self.ui_state.dialog_stack.pop()
    }

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

    pub fn save_config(&mut self) -> Result<()> {
        let config = self.config.lock().unwrap().clone();
        let mut storage = self.storage.lock().unwrap();
        storage.update_config(config.storage)?;
        Ok(())
    }
}

#[cfg(target_os = "windows")]
fn is_dark_mode_enabled() -> bool {
    use winreg::enums::HKEY_CURRENT_USER;
    use winreg::RegKey;

    if let Ok(hkcu) = RegKey::predef(HKEY_CURRENT_USER).open_subkey(
        "Software\\Microsoft\\Windows\\CurrentVersion\\Themes\\Personalize"
    ) {
        if let Ok(dark_mode) = hkcu.get_value::<u32, _>("AppsUseLightTheme") {
            return dark_mode == 0;
        }
    }
    false
}

#[cfg(target_os = "macos")]
fn is_dark_mode_enabled() -> bool {
    false
}

#[cfg(not(any(target_os = "windows", target_os = "macos")))]
fn is_dark_mode_enabled() -> bool {
    false
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    impl TimeTrackerApp {
        pub fn test_new() -> Self {
            use std::sync::mpsc::channel;
            let (tx, rx) = channel();
            let temp_dir = TempDir::new().unwrap();
            let config = Config::default();
            
            Self::new(
                Arc::new(Mutex::new(config.clone())),
                Arc::new(Mutex::new(Storage::new_in_memory().unwrap())),
                Arc::new(Mutex::new(PomodoroTimer::new(Default::default(), Default::default()))),
                Arc::new(Mutex::new(AppTracker::new(Default::default())
                    .expect("Failed to create AppTracker"))),
                Arc::new(Mutex::new(AppStateManager::new(
                    temp_dir.path().to_path_buf(),
                    true,
                ).unwrap())),
                Arc::new(Mutex::new(TrayManager::new(
                    temp_dir.path().join("icon.png"),
                    tx.clone()
                ).unwrap())),
                Arc::new(Mutex::new(HotkeyManager::new(config.general.hotkeys))),
                rx,
            )
        }
    }

    #[test]
    fn test_dialog_stack() {
        use crate::ui::dialogs::ConfirmationDialog;
        
        let mut app = TimeTrackerApp::test_new();
        assert!(app.ui_state.dialog_stack.is_empty());
        
        app.push_dialog(Box::new(ConfirmationDialog {
            title: "Test".to_string(),
            message: "Test message".to_string(),
            on_confirm: None,
            on_cancel: None,
        }));
        
        assert_eq!(app.ui_state.dialog_stack.len(), 1);
        
        let dialog = app.pop_dialog();
        assert!(dialog.is_some());
        assert!(app.ui_state.dialog_stack.is_empty());
    }
}