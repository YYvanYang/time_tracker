use tray_item::{IconSource, TrayItem};
use crate::core::AppResult;

pub struct SystemTray {
    inner: TrayItem,
}

impl SystemTray {
    pub fn new() -> AppResult<Self> {
        let tray = TrayItem::new("Time Tracker", IconSource::Resource("app-icon"))?;
        Ok(Self { inner: tray })
    }

    pub fn set_icon(&mut self, icon_name: &str) -> AppResult<()> {
        self.inner.set_icon(IconSource::Resource(icon_name))?;
        Ok(())
    }

    pub fn set_tooltip(&mut self, title: &str) -> AppResult<()> {
        self.inner.set_tooltip(title)?;
        Ok(())
    }

    pub fn add_menu_item<F>(&mut self, label: &str, handler: F) -> AppResult<()>
    where
        F: Fn() + Send + Sync + 'static,
    {
        self.inner.add_menu_item(label, Box::new(handler))?;
        Ok(())
    }
} 