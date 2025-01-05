// src/ui/mod.rs

mod components;
mod views;
mod styles;
mod theme;

use crate::error::Result;
use crate::config::Config;
use crate::storage::Storage;
use crate::app_tracker::AppTracker;
use crate::pomodoro::PomodoroTimer;
use crate::storage::app_state::AppStateManager;
use components::*;
use eframe::egui;
use std::sync::Arc;
use views::*;
use crate::ui::components::Dialog;
use crate::ui::components::dialog::{ProjectDialog, TagDialog, ExportDialog, SettingsDialog, ConfirmationDialog};
use parking_lot::Mutex;

pub struct TimeTrackerApp {
    config: Arc<Mutex<Config>>,
    storage: Arc<Storage>,
    app_tracker: Arc<AppTracker>,
    pomodoro: Arc<PomodoroTimer>,
    state_manager: Arc<AppStateManager>,
    current_view: View,
    dialog_stack: Vec<Dialog>,
    loading: bool,
    error: Option<String>,
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

#[derive(Debug)]
pub enum Dialog {
    AddProject(ProjectDialog),
    EditProject(ProjectDialog),
    AddTag(TagDialog),
    Export(ExportDialog),
    Settings(SettingsDialog),
    Confirmation(ConfirmationDialog),
}

impl TimeTrackerApp {
    pub fn new(
        config: Config,
        storage: Storage,
        app_tracker: AppTracker,
        pomodoro: PomodoroTimer,
        state_manager: AppStateManager,
    ) -> Self {
        Self {
            config: Arc::new(Mutex::new(config)),
            storage: Arc::new(storage),
            app_tracker: Arc::new(app_tracker),
            pomodoro: Arc::new(pomodoro),
            state_manager: Arc::new(state_manager),
            current_view: View::Overview,
            dialog_stack: Vec::new(),
            loading: false,
            error: None,
        }
    }

    fn show_error(&mut self, error: String) {
        log::error!("UI Error: {}", error);
        self.error = Some(error);
    }

    fn clear_error(&mut self) {
        self.error = None;
    }

    fn push_dialog(&mut self, dialog: Dialog) {
        self.dialog_stack.push(dialog);
    }

    fn pop_dialog(&mut self) -> Option<Dialog> {
        self.dialog_stack.pop()
    }

    fn show_confirmation(
        &mut self,
        title: String,
        message: String,
        on_confirm: Box<dyn FnOnce(&mut Self) -> Result<()>>,
    ) {
        self.push_dialog(Dialog::Confirmation(ConfirmationDialog {
            title,
            message,
            on_confirm: Some(on_confirm),
            on_cancel: None,
        }));
    }

    pub fn show_confirmation_dialog(&mut self, title: String, message: String, on_confirm: Box<dyn FnOnce()>) {
        self.push_dialog(Dialog::Confirmation(ConfirmationDialog {
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
        let config = self.config.lock();
        config.save()?;
        Ok(())
    }

    pub fn get_config(&self) -> parking_lot::MutexGuard<Config> {
        self.config.lock()
    }
}

impl eframe::App for TimeTrackerApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // 更新主题
        theme::apply_theme(ctx, &self.config);

        // 显示顶部菜单栏
        self.show_top_panel(ctx);

        // 显示侧边栏（如果不是紧凑模式）
        if !self.config.lock().ui.compact_mode {
            self.show_sidebar(ctx);
        }

        // 显示主内容区域
        egui::CentralPanel::default().show(ctx, |ui| {
            // 显示错误信息（如果有）
            if let Some(error) = &self.error {
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
            match self.current_view {
                View::Overview => overview::render(self, ui),
                View::AppUsage => app_usage::render(self, ui),
                View::Pomodoro => pomodoro::render(self, ui),
                View::Projects => projects::render(self, ui),
                View::Statistics => statistics::render(self, ui),
                View::Settings => settings::render(self, ui),
            }
        });

        // 显示对话框
        if !self.dialog_stack.is_empty() {
            self.show_dialogs(ctx);
        }

        // 显示加载指示器
        if self.loading {
            self.show_loading_indicator(ctx);
        }

        // 定期更新
        ctx.request_repaint_after(std::time::Duration::from_secs(1));
    }

    fn save(&mut self, _storage: &mut dyn eframe::Storage) {
        // 保存窗口状态
        if let Ok(state) = self.state_manager.get_state() {
            if let Err(e) = self.state_manager.save() {
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
                        self.push_dialog(Dialog::Export(ExportDialog::default()));
                        ui.close_menu();
                    }
                    if ui.button("设置").clicked() {
                        self.push_dialog(Dialog::Settings(SettingsDialog::new(&self.config)));
                        ui.close_menu();
                    }
                });

                ui.menu_button("视图", |ui| {
                    if ui.button("概览").clicked() {
                        self.current_view = View::Overview;
                        ui.close_menu();
                    }
                    if ui.button("应用统计").clicked() {
                        self.current_view = View::AppUsage;
                        ui.close_menu();
                    }
                    if ui.button("番茄钟").clicked() {
                        self.current_view = View::Pomodoro;
                        ui.close_menu();
                    }
                    if ui.button("项目").clicked() {
                        self.current_view = View::Projects;
                        ui.close_menu();
                    }
                    if ui.button("统计").clicked() {
                        self.current_view = View::Statistics;
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
        egui::SidePanel::left("side_panel")
            .resizable(false)
            .default_width(200.0)
            .show(ctx, |ui| {
                ui.vertical(|ui| {
                    ui.selectable_value(
                        &mut self.current_view,
                        View::Overview,
                        "概览",
                    );
                    ui.selectable_value(
                        &mut self.current_view,
                        View::AppUsage,
                        "应用统计",
                    );
                    ui.selectable_value(
                        &mut self.current_view,
                        View::Pomodoro,
                        "番茄钟",
                    );
                    ui.selectable_value(
                        &mut self.current_view,
                        View::Projects,
                        "项目",
                    );
                    ui.selectable_value(
                        &mut self.current_view,
                        View::Statistics,
                        "统计",
                    );
                    ui.selectable_value(
                        &mut self.current_view,
                        View::Settings,
                        "设置",
                    );
                });
            });
    }

    fn show_dialogs(&mut self, ctx: &egui::Context) {
        use crate::ui::components::dialog::Dialog;
        
        if let Some(dialog) = self.dialog_stack.last_mut() {
            match dialog {
                Dialog::Project(dialog) => {
                    dialog.show(ctx, self);
                }
                Dialog::Tag(dialog) => {
                    dialog.show(ctx, self);
                }
                Dialog::Export(dialog) => {
                    dialog.show(ctx, self);
                }
                Dialog::Settings(dialog) => {
                    dialog.show(ctx, self);
                }
                Dialog::Confirmation(dialog) => {
                    dialog.show(ctx, self);
                }
            }
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
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Config;
    use tempfile::TempDir;

    fn create_test_app() -> (TimeTrackerApp, TempDir) {
        let temp_dir = TempDir::new().unwrap();
        let config = Config::default();
        let storage = Storage::new(config.storage.clone()).unwrap();
        let app_tracker = AppTracker::new(Default::default());
        let pomodoro = PomodoroTimer::new(Default::default(), Default::default());
        let state_manager = AppStateManager::new(
            temp_dir.path().to_path_buf(),
            true,
        ).unwrap();

        let app = TimeTrackerApp::new(
            config,
            storage,
            app_tracker,
            pomodoro,
            state_manager,
        );

        (app, temp_dir)
    }

    #[test]
    fn test_dialog_management() {
        let (mut app, _temp_dir) = create_test_app();

        assert_eq!(app.current_view, View::Overview);

        // 测试视图切换
        app.current_view = View::Pomodoro;
        assert_eq!(app.current_view, View::Pomodoro);

        app.current_view = View::Statistics;
        assert_eq!(app.current_view, View::Statistics);
    }

    #[test]
    fn test_error_management() {
        let (mut app, _temp_dir) = create_test_app();

        assert!(app.error.is_none());

        app.show_error("Test error".to_string());
        assert_eq!(app.error, Some("Test error".to_string()));

        app.clear_error();
        assert!(app.error.is_none());
    }

    #[test]
    fn test_view_switching() {
        let (mut app, _temp_dir) = create_test_app();

        // 测试对话框堆栈
        assert!(app.dialog_stack.is_empty());

        app.push_dialog(Dialog::Confirmation(ConfirmationDialog {
            title: "Test".to_string(),
            message: "Test message".to_string(),
            on_confirm: None,
            on_cancel: None,
        }));

        assert_eq!(app.dialog_stack.len(), 1);

        let dialog = app.pop_dialog();
        assert!(matches!(dialog, Some(Dialog::Confirmation(_))));
        assert!(app.dialog_stack.is_empty());
    }
}