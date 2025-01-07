use crate::core::AppResult;
use crate::presentation::ui::TimeTrackerApp;
use std::sync::Arc;
use tray_item::TrayItem;

pub struct TrayIcon {
    inner: TrayItem,
    app: Arc<TimeTrackerApp>,
}

impl TrayIcon {
    pub fn new(app: Arc<TimeTrackerApp>) -> AppResult<Self> {
        let mut tray = TrayItem::new("Time Tracker", "time-tracker-tray")?;

        tray.add_menu_item("显示主窗口", {
            let app = Arc::clone(&app);
            move || {
                if let Some(window) = app.window() {
                    window.show();
                }
            }
        })?;

        tray.add_menu_item("开始番茄钟", {
            let app = Arc::clone(&app);
            move || {
                app.start_pomodoro();
            }
        })?;

        tray.add_menu_item("暂停番茄钟", {
            let app = Arc::clone(&app);
            move || {
                app.pause_pomodoro();
            }
        })?;

        tray.add_menu_item("停止番茄钟", {
            let app = Arc::clone(&app);
            move || {
                app.stop_pomodoro();
            }
        })?;

        tray.add_menu_item("退出", {
            let app = Arc::clone(&app);
            move || {
                app.quit();
            }
        })?;

        Ok(Self { inner: tray, app })
    }

    pub fn update_icon(&mut self, icon_name: &str) -> AppResult<()> {
        self.inner.set_icon(icon_name)?;
        Ok(())
    }

    pub fn update_tooltip(&mut self, tooltip: &str) -> AppResult<()> {
        self.inner.set_tooltip(tooltip)?;
        Ok(())
    }
} 