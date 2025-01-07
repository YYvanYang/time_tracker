use crate::core::AppResult;
use crate::presentation::ui::{TimeTrackerApp, View};
use eframe::egui;

pub struct MainWindow {
    app: TimeTrackerApp,
}

impl MainWindow {
    pub fn new(app: TimeTrackerApp) -> Self {
        Self { app }
    }

    pub fn run(app: TimeTrackerApp) -> AppResult<()> {
        let options = eframe::NativeOptions {
            initial_window_size: Some(egui::vec2(800.0, 600.0)),
            min_window_size: Some(egui::vec2(400.0, 300.0)),
            centered: true,
            ..Default::default()
        };

        eframe::run_native(
            "Time Tracker",
            options,
            Box::new(|_cc| Box::new(MainWindow::new(app))),
        )?;

        Ok(())
    }
}

impl eframe::App for MainWindow {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                ui.menu_button("文件", |ui| {
                    if ui.button("设置").clicked() {
                        self.app.set_view(View::Settings);
                        ui.close_menu();
                    }
                    if ui.button("退出").clicked() {
                        _frame.close();
                    }
                });

                ui.menu_button("视图", |ui| {
                    if ui.button("概览").clicked() {
                        self.app.set_view(View::Overview);
                        ui.close_menu();
                    }
                    if ui.button("应用使用").clicked() {
                        self.app.set_view(View::AppUsage);
                        ui.close_menu();
                    }
                    if ui.button("番茄钟").clicked() {
                        self.app.set_view(View::Pomodoro);
                        ui.close_menu();
                    }
                    if ui.button("项目").clicked() {
                        self.app.set_view(View::Projects);
                        ui.close_menu();
                    }
                    if ui.button("统计").clicked() {
                        self.app.set_view(View::Statistics);
                        ui.close_menu();
                    }
                });

                ui.menu_button("帮助", |ui| {
                    if ui.button("关于").clicked() {
                        // TODO: 显示关于对话框
                        ui.close_menu();
                    }
                });
            });
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            match self.app.current_view() {
                View::Overview => self.app.render_overview(ui),
                View::AppUsage => self.app.render_app_usage(ui),
                View::Pomodoro => self.app.render_pomodoro(ui),
                View::Projects => self.app.render_projects(ui),
                View::Statistics => self.app.render_statistics(ui),
                View::Settings => self.app.render_settings(ui),
            }
        });

        // 处理对话框
        if let Some(dialog) = self.app.current_dialog() {
            dialog.show(ctx, &mut self.app);
        }
    }
} 