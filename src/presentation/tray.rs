use std::sync::Arc;
use tray_item::TrayItem;
use crate::core::AppResult;

pub struct TrayManager {
    tray: Arc<TrayItem>,
}

impl TrayManager {
    pub fn new() -> AppResult<Self> {
        let tray = TrayItem::new("Time Tracker", "time-tracker-tray")?;
        
        Ok(Self {
            tray: Arc::new(tray),
        })
    }

    pub fn show(&self) -> AppResult<()> {
        Ok(())
    }

    pub fn hide(&self) -> AppResult<()> {
        Ok(())
    }

    pub fn set_icon(&self, icon_path: &str) -> AppResult<()> {
        self.tray.set_icon(icon_path)?;
        Ok(())
    }

    pub fn set_tooltip(&self, tooltip: &str) -> AppResult<()> {
        self.tray.set_tooltip(tooltip)?;
        Ok(())
    }
} 