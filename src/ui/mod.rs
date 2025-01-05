mod components;
mod views;
mod styles;
mod theme;

use crate::config::Theme as AppTheme;
use crate::error::Result;
use crate::config::Config;
use crate::storage::Storage;
use crate::app_tracker::AppTracker;
use crate::pomodoro::PomodoroTimer;
use crate::storage::app_state::AppStateManager;
use crate::tray::{TrayManager, TrayEvent};
use crate::hotkeys::HotkeyManager;
use eframe::egui;
use std::sync::{Arc, Mutex, mpsc::Receiver};
use views::*;
use crate::ui::components::Dialog as ComponentDialog;
use crate::ui::components::dialog::{ProjectDialog, TagDialog, ExportDialog, SettingsDialog, ConfirmationDialog};
use std::collections::{HashSet, VecDeque, HashMap};
use crate::storage::{Project, Tag, PomodoroStatus};
use crate::error::TimeTrackerError;
use crate::storage::app_state::AppState;

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
        }
    }

    fn show_error(&mut self, error: String) {
        log::error!("UI Error: {}", error);
        self.ui_state.error = Some(error);
    }

    fn clear_error(&mut self) {
        self.ui_state.error = None;
    }

    fn push_dialog(&mut self, dialog: Box<dyn DialogHandler + Send>) {
        self.ui_state.dialog_stack.push(dialog);
    }

    fn pop_dialog(&mut self) -> Option<Box<dyn DialogHandler + Send>> {
        self.ui_state.dialog_stack.pop()
    }

    fn show_confirmation(
        &mut self,
        title: String,
        message: String,
        on_confirm: Box<dyn FnOnce(&mut Self) -> Result<()> + Send>,
    ) {
        self.push_dialog(Box::new(ConfirmationDialog {
            title,
            message,
            on_confirm: Some(on_confirm),
            on_cancel: None,
        }));
    }

    pub fn show_confirmation_dialog(
        &mut self, 
        title: String, 
        message: String, 
        on_confirm: Box<dyn FnOnce() + Send>
    ) {
        self.push_dialog(Box::new(ConfirmationDialog {
            title,
            message,
            on_confirm: Some(Box::new(move |_| {
                on_confirm();
                Ok(())
            })),
            on_cancel: None,
        }));
    }

    pub fn save_config(&mut self) -> Result<()> {
        let mut config = self.config.lock()?;
        config.save()?;
        Ok(())
    }

    pub fn get_config(&self) -> std::sync::MutexGuard<Config> {
        self.config.lock().unwrap()
    }

    pub fn get_current_project(&self) -> Option<&Project> {
        self.current_project.as_ref()
    }

    fn save_state(&mut self) -> Result<()> {
        // 先获取错误信息
        let save_result = {
            let mut state_manager = self.app_state_manager.lock()?;
            state_manager.save_state()
        };

        // 然后处理错误
        if let Err(e) = save_result {
            log::error!("Failed to save app state: {}", e);
            self.show_error(format!("Failed to save app state: {}", e));
        }
        Ok(())
    }
}

impl eframe::App for TimeTrackerApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // 获取主题设置
        let is_dark = match self.config.lock().unwrap().ui.theme {
            AppTheme::Dark => true,
            AppTheme::Light => false,
            AppTheme::System => is_dark_mode_enabled(),
        };
        
        theme::apply_theme(ctx, is_dark);

        // 显示顶部菜单栏
        self.show_top_panel(ctx);

        // 显示侧边栏（如果不是紧凑模式）
        if !self.config.lock().unwrap().ui.compact_mode {
            self.show_sidebar(ctx);
        }

        // 显示主内容区域
        egui::CentralPanel::default().show(ctx, |ui| {
            // 显示错误信息（如果有）
            if let Some(error) = &self.ui_state.error {
                ui.colored_label(
                    styles::COLOR_ERROR,
                    error,
                );
                if ui.button("关闭").clicked() {
                    self.clear_error();
                }
                ui.separator();
            }

            // 显示当前视图
            match self.ui_state.current_view {
                View::Overview => overview::render(self, ui),
                View::AppUsage => app_usage::render(self, ui),
                View::Pomodoro => pomodoro::render(self, ui),
                View::Projects => projects::render(self, ui),
                View::Statistics => statistics::render(self, ui),
                View::Settings => settings::render(self, ui),
            }
        });

        // 显示对话框
        if !self.ui_state.dialog_stack.is_empty() {
            self.show_dialogs(ctx);
        }

        // 显示加载指示器
        if self.ui_state.loading {
            self.show_loading_indicator(ctx);
        }

        // 定期更新
        ctx.request_repaint_after(std::time::Duration::from_secs(1));
    }

    fn save(&mut self, _storage: &mut dyn eframe::Storage) {
        // 保存窗口状态
        if let Ok(mut state_manager) = self.app_state_manager.lock() {
            if let Err(e) = state_manager.save_state() {
                log::error!("Failed to save app state: {}", e);
            }
        }
    }
}

// UI辅助方法实现
impl TimeTrackerApp {
    fn show_top_panel(&mut self, ctx: &egui::Context) {
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.menu_button("文件", |ui| {
                    if ui.button("导出数据").clicked() {
                        self.push_dialog(Box::new(ExportDialog::default()));
                        ui.close_menu();
                    }
                    if ui.button("设置").clicked() {
                        let config = {
                            let config_guard = self.config.lock().unwrap();
                            SettingsDialog::new(&*config_guard)
                        };
                        self.push_dialog(Box::new(config));
                        ui.close_menu();
                    }
                });

                ui.menu_button("视图", |ui| {
                    if ui.button("概览").clicked() {
                        self.ui_state.current_view = View::Overview;
                        ui.close_menu();
                    }
                    if ui.button("应用统计").clicked() {
                        self.ui_state.current_view = View::AppUsage;
                        ui.close_menu();
                    }
                    if ui.button("番茄钟").clicked() {
                        self.ui_state.current_view = View::Pomodoro;
                        ui.close_menu();
                    }
                    if ui.button("项目").clicked() {
                        self.ui_state.current_view = View::Projects;
                        ui.close_menu();
                    }
                    if ui.button("统计").clicked() {
                        self.ui_state.current_view = View::Statistics;
                        ui.close_menu();
                    }
                });

                ui.menu_button("帮助", |ui| {
                    if ui.button("关于").clicked() {
                        // TODO: 显示关于对话框
                        ui.close_menu();
                    }
                    if ui.button("检查更新").clicked() {
                        // TODO: 检查更新
                        ui.close_menu();
                    }
                });

                // 右侧显示时间和状态
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.label(chrono::Local::now().format("%H:%M:%S").to_string());
                });
            });
        });
    }

    fn show_sidebar(&mut self, ctx: &egui::Context) {
        let config = self.config.lock().unwrap();
        if !config.ui.compact_mode {
            egui::SidePanel::left("side_panel")
                .resizable(false)
                .default_width(200.0)
                .show(ctx, |ui| {
                    ui.vertical(|ui| {
                        ui.selectable_value(
                            &mut self.ui_state.current_view,
                            View::Overview,
                            "概览",
                        );
                        ui.selectable_value(
                            &mut self.ui_state.current_view,
                            View::AppUsage,
                            "应用统计",
                        );
                        ui.selectable_value(
                            &mut self.ui_state.current_view,
                            View::Pomodoro,
                            "番茄钟",
                        );
                        ui.selectable_value(
                            &mut self.ui_state.current_view,
                            View::Projects,
                            "项目",
                        );
                        ui.selectable_value(
                            &mut self.ui_state.current_view,
                            View::Statistics,
                            "统计",
                        );
                        ui.selectable_value(
                            &mut self.ui_state.current_view,
                            View::Settings,
                            "设置",
                        );
                    });
                });
        }
    }

    fn show_dialogs(&mut self, ctx: &egui::Context) {
        let mut dialog_closed = false;
        
        // 先克隆需要的数据
        let config = self.config.clone();
        let app_state_manager = self.app_state_manager.clone();
        let storage = self.storage.clone();
        
        if let Some(dialog) = self.ui_state.dialog_stack.last_mut() {
            let config_guard = config.lock().unwrap();
            let state_manager_guard = app_state_manager.lock().unwrap();
            let state = state_manager_guard.get_state().unwrap();
            let storage_guard = storage.lock().unwrap();
            
            let mut dialog_context = DialogContext {
                config: &*config_guard,
                state: &state,
                storage: &*storage_guard,
            };

            if !dialog.show(ctx, &mut dialog_context) {
                dialog_closed = true;
            }
        }
        
        if dialog_closed {
            self.ui_state.dialog_stack.pop();
        }
    }

    fn show_loading_indicator(&self, ctx: &egui::Context) {
        egui::Window::new("加载中")
            .collapsible(false)
            .resizable(false)
            .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 0.0))
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.spinner();
                    ui.label("请稍候...");
                });
            });
    }

    fn show_dialog(&mut self, ctx: &egui::Context) {
        let dialog = match self.ui_state.dialog_stack.last_mut() {
            Some(dialog) => dialog,
            None => return,
        };

        let config_guard = self.config.lock().unwrap();
        let state_manager_guard = self.app_state_manager.lock().unwrap();
        let state = state_manager_guard.get_state().unwrap();
        let storage_guard = self.storage.lock().unwrap();

        let mut dialog_context = DialogContext {
            config: &*config_guard,
            state: &state,
            storage: &*storage_guard,
        };

        if !dialog.show(ctx, &mut dialog_context) {
            self.ui_state.dialog_stack.pop();
        }
    }

    fn push_settings_dialog(&mut self) {
        let dialog = {
            let config = self.config.lock().unwrap();
            SettingsDialog::new(&*config)
        };
        self.push_dialog(Box::new(dialog));
    }
}

impl std::fmt::Debug for TimeTrackerApp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TimeTrackerApp")
            .field("current_view", &self.ui_state.current_view)
            .field("loading", &self.ui_state.loading)
            .field("error", &self.ui_state.error)
            .field("dialog_count", &self.ui_state.dialog_stack.len())
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Config;
    use tempfile::TempDir;

    fn create_test_app() -> (TimeTrackerApp, TempDir) {
        use std::sync::mpsc::channel;
        
        let (tx, rx) = channel();
        let temp_dir = TempDir::new().unwrap();
        let config = Config::default();
        
        let app = TimeTrackerApp::new(
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
        );

        (app, temp_dir)
    }

    #[test]
    fn test_dialog_management() {
        let (mut app, _temp_dir) = create_test_app();

        assert_eq!(app.ui_state.current_view, View::Overview);

        // 测试视图切换
        app.ui_state.current_view = View::Pomodoro;
        assert_eq!(app.ui_state.current_view, View::Pomodoro);

        app.ui_state.current_view = View::Statistics;
        assert_eq!(app.ui_state.current_view, View::Statistics);
    }

    #[test]
    fn test_error_management() {
        let (mut app, _temp_dir) = create_test_app();

        assert!(app.ui_state.error.is_none());

        app.show_error("Test error".to_string());
        assert_eq!(app.ui_state.error, Some("Test error".to_string()));

        app.clear_error();
        assert!(app.ui_state.error.is_none());
    }

    #[test]
    fn test_view_switching() {
        let (mut app, _temp_dir) = create_test_app();
        
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

// 修改 DialogContext 的定义，使用 MutexGuard
pub struct DialogContext<'a> {
    pub config: &'a Config,
    pub state: &'a AppState,
    pub storage: &'a Storage,
}

// 修改 DialogHandler trait 以适应新的 DialogContext
pub trait DialogHandler {
    fn show(&mut self, ctx: &egui::Context, dialog_ctx: &mut DialogContext) -> bool;
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
    // macOS 实现...
    false
}

#[cfg(not(any(target_os = "windows", target_os = "macos")))]
fn is_dark_mode_enabled() -> bool {
    false
}

#[cfg(test)]
impl TimeTrackerApp {
    pub fn test_new() -> Self {
        use crate::config::Config;
        use crate::storage::Storage;
        use crate::pomodoro::PomodoroTimer;
        use crate::app_tracker::AppTracker;
        use crate::storage::app_state::AppStateManager;
        use crate::tray::TrayManager;
        use crate::hotkeys::HotkeyManager;
        use std::sync::{Arc, Mutex};
        use std::sync::mpsc::channel;
        use tempfile::TempDir;

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