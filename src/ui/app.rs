use crate::error::Result;
use crate::storage::Storage;
use crate::pomodoro::PomodoroTimer;
use crate::app_tracker::AppTracker;
use crate::storage::app_state::AppStateManager;
use crate::tray::TrayManager;
use crate::hotkeys::HotkeyManager;
use crate::storage::models::Project;

use std::sync::Arc;
use std::sync::Mutex;
use std::sync::mpsc::Receiver;
use eframe::egui;
use std::future::Future;
use std::pin::Pin;

pub struct TimeTrackerApp {
    config: crate::config::Config,
    storage: Arc<Mutex<Storage>>,
    pomodoro_timer: Arc<Mutex<PomodoroTimer>>,
    app_tracker: Arc<Mutex<AppTracker>>,
    app_state_manager: Arc<Mutex<AppStateManager>>,
    tray_manager: Arc<Mutex<TrayManager>>,
    hotkey_manager: Arc<Mutex<HotkeyManager>>,
    tray_event_receiver: Receiver<crate::tray::TrayEvent>,
    current_project: Option<Project>,
}

impl TimeTrackerApp {
    pub fn new(
        config: crate::config::Config,
        storage: Arc<Mutex<Storage>>,
        pomodoro_timer: Arc<Mutex<PomodoroTimer>>,
        app_tracker: Arc<Mutex<AppTracker>>,
        app_state_manager: Arc<Mutex<AppStateManager>>,
        tray_manager: Arc<Mutex<TrayManager>>,
        hotkey_manager: Arc<Mutex<HotkeyManager>>,
        tray_event_receiver: Receiver<crate::tray::TrayEvent>,
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
        }
    }

    pub fn get_current_project(&self) -> Option<&Project> {
        self.current_project.as_ref()
    }

    pub fn spawn_task(&self, future: Pin<Box<dyn Future<Output = ()> + Send>>) {
        tokio::spawn(future);
    }
}

impl eframe::App for TimeTrackerApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // UI 更新逻辑
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Time Tracker");
            // 添加更多 UI 元素...
        });
    }
} 