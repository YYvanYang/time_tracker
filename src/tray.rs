//src/tray.rs

use crate::error::Result;
use std::sync::mpsc;
use tray_item::{IconSource, TrayItem};

pub enum TrayEvent {
    Show,
    Hide,
    StartPomodoro,
    PausePomodoro,
    StopPomodoro,
    Exit,
}

pub struct TrayManager {
    tray: TrayItem,
    event_sender: mpsc::Sender<TrayEvent>,
    event_receiver: mpsc::Receiver<TrayEvent>,
}

impl TrayManager {
    pub fn new() -> Result<Self> {
        let (tx, rx) = mpsc::channel();
        let sender = tx.clone();

        let mut tray = TrayItem::new(
            "Time Tracker",
            IconSource::Resource("tray-icon"), // 需要在资源文件中定义图标
        )?;

        // 设置托盘菜单
        tray.add_menu_item("显示主窗口", move || {
            sender.send(TrayEvent::Show).ok();
        })?;

        let sender = tx.clone();
        tray.add_menu_item("开始番茄钟", move || {
            sender.send(TrayEvent::StartPomodoro).ok();
        })?;

        let sender = tx.clone();
        tray.add_menu_item("暂停番茄钟", move || {
            sender.send(TrayEvent::PausePomodoro).ok();
        })?;

        let sender = tx.clone();
        tray.add_menu_item("停止番茄钟", move || {
            sender.send(TrayEvent::StopPomodoro).ok();
        })?;

        tray.add_menu_separator()?;

        let sender = tx.clone();
        tray.add_menu_item("退出", move || {
            sender.send(TrayEvent::Exit).ok();
        })?;

        Ok(Self {
            tray,
            event_sender: tx,
            event_receiver: rx,
        })
    }

    pub fn set_icon(&mut self, icon_name: &str) -> Result<()> {
        self.tray.set_icon(IconSource::Resource(icon_name))?;
        Ok(())
    }

    pub fn set_tooltip(&mut self, tooltip: &str) -> Result<()> {
        self.tray.set_tooltip(tooltip)?;
        Ok(())
    }

    pub fn update_pomodoro_status(&mut self, status: &str) -> Result<()> {
        // 更新图标和提示文本
        match status {
            "工作中" => {
                self.set_icon("tray-icon-work")?;
                self.set_tooltip(&format!("Time Tracker - {}", status))?;
            }
            "休息中" => {
                self.set_icon("tray-icon-break")?;
                self.set_tooltip(&format!("Time Tracker - {}", status))?;
            }
            _ => {
                self.set_icon("tray-icon")?;
                self.set_tooltip("Time Tracker")?;
            }
        }
        Ok(())
    }

    pub fn get_event_receiver(&self) -> mpsc::Receiver<TrayEvent> {
        self.event_receiver.clone()
    }

    pub fn send_event(&self, event: TrayEvent) -> Result<()> {
        self.event_sender.send(event).map_err(|e| {
            crate::error::TimeTrackerError::Platform(
                format!("Failed to send tray event: {}", e)
            )
        })?;
        Ok(())
    }
}

// 系统托盘图标资源
#[cfg(windows)]
mod resources {
    use winres::WindowsResource;

    pub fn build_resources() {
        let mut res = WindowsResource::new();
        res.set_icon("assets/icons/tray-icon.ico");
        res.set_icon_with_id("assets/icons/tray-icon-work.ico", "tray-icon-work");
        res.set_icon_with_id("assets/icons/tray-icon-break.ico", "tray-icon-break");
        res.compile().unwrap();
    }
}

#[cfg(unix)]
mod resources {
    pub fn build_resources() {
        // Linux/macOS 使用文件图标
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn test_tray_events() {
        let tray = TrayManager::new().unwrap();
        let receiver = tray.get_event_receiver();

        // 发送事件
        tray.send_event(TrayEvent::Show).unwrap();

        // 接收事件
        if let Ok(event) = receiver.recv_timeout(Duration::from_secs(1)) {
            match event {
                TrayEvent::Show => assert!(true),
                _ => assert!(false, "Received wrong event"),
            }
        } else {
            assert!(false, "No event received");
        }
    }

    #[test]
    fn test_tray_status_update() {
        let mut tray = TrayManager::new().unwrap();
        
        // 测试不同状态的图标和提示文本更新
        assert!(tray.update_pomodoro_status("工作中").is_ok());
        assert!(tray.update_pomodoro_status("休息中").is_ok());
        assert!(tray.update_pomodoro_status("空闲").is_ok());
    }
}