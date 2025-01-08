use tray_item::TrayItem;
use crate::core::AppResult;

pub struct TrayIcon {
    inner: TrayItem,
}

impl TrayIcon {
    pub fn new() -> AppResult<Self> {
        let tray = TrayItem::new("Time Tracker", "app-icon")?;
        Ok(Self { inner: tray })
    }

    pub fn set_icon(&mut self, icon_name: &str) -> AppResult<()> {
        self.inner.set_icon(icon_name)?;
        Ok(())
    }

    pub fn set_title(&mut self, title: &str) -> AppResult<()> {
        self.inner.set_title(title)?;
        Ok(())
    }

    pub fn add_menu_item(&mut self, label: &str, handler: Box<dyn Fn() + Send + 'static>) -> AppResult<()> {
        self.inner.add_menu_item(label, handler)?;
        Ok(())
    }
} 