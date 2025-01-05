use crate::error::{Result, TimeTrackerError};
use std::sync::mpsc::{self, Sender};
use std::path::PathBuf;
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
    event_sender: Sender<TrayEvent>,
    icon_base_path: String,
}

impl TrayManager {
    pub fn new(icon_path: PathBuf, event_sender: Sender<TrayEvent>) -> Result<Self> {
        let icon_base_path = icon_path.to_str()
            .ok_or_else(|| TimeTrackerError::Platform("Invalid icon path".into()))?
            .to_string();
        
        let icon_path_str = icon_base_path.clone();
        
        let tray = TrayItem::new(
            "Time Tracker",
            IconSource::Resource(Box::leak(icon_path_str.into_boxed_str()))
        )?;
        
        Ok(Self {
            tray,
            event_sender,
            icon_base_path,
        })
    }

    pub fn set_icon(&mut self, icon_name: &str) -> Result<()> {
        let icon_str = Box::leak(icon_name.to_string().into_boxed_str());
        self.tray.set_icon(IconSource::Resource(icon_str))?;
        Ok(())
    }

    pub fn set_tooltip(&mut self, _tooltip: &str) -> Result<()> {
        // 如果平台不支持 tooltip，返回 Ok
        Ok(())
    }

    pub fn update_pomodoro_status(&mut self, status: &str) -> Result<()> {
        // 更新图标和提示文本
        let icon_path = match status {
            "工作中" => format!("{}_work", self.icon_base_path),
            "休息中" => format!("{}_break", self.icon_base_path),
            _ => self.icon_base_path.clone(),
        };

        self.set_icon(&icon_path)?;
        self.set_tooltip(&format!("Time Tracker - {}", status))?;
        Ok(())
    }

    pub fn get_event_receiver(&self) -> mpsc::Receiver<TrayEvent> {
        let (_tx, rx) = mpsc::channel();
        self.event_sender.send(TrayEvent::Show).ok();
        rx
    }

    pub fn send_event(&self, event: TrayEvent) -> Result<()> {
        self.event_sender.send(event).map_err(|e| {
            TimeTrackerError::Platform(format!("Failed to send tray event: {}", e))
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
        // 创建一个通道来接收事件
        let (sender, receiver) = std::sync::mpsc::channel();
        
        // 创建一个临时图标路径
        let icon_path = std::path::PathBuf::from("resources/app_icon.png");
        
        // 使用正确的参数创建 TrayManager
        let tray = TrayManager::new(icon_path, sender).unwrap();

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
        // 创建一个通道来接收事件
        let (sender, _receiver) = std::sync::mpsc::channel();
        
        // 创建一个临时图标路径
        let icon_path = std::path::PathBuf::from("resources/app_icon.png");
        
        // 使用正确的参数创建 TrayManager
        let mut tray = TrayManager::new(icon_path, sender).unwrap();
        
        // 测试不同状态的图标和提示文本更新
        assert!(tray.update_pomodoro_status("工作中").is_ok());
        assert!(tray.update_pomodoro_status("休息中").is_ok());
        assert!(tray.update_pomodoro_status("空闲").is_ok());
    }
}