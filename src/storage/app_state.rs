// src/storage/app_state.rs

use crate::error::{Result, TimeTrackerError};
use crate::pomodoro::{PomodoroState, PomodoroConfig};
use crate::app_tracker::AppUsageConfig;
use chrono::{DateTime, Local};
use serde::{Serialize, Deserialize};
use std::sync::{Arc, RwLock};
use std::time::Duration;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppState {
    #[serde(skip)]
    pub is_modified: bool,
    pub last_sync: Option<DateTime<Local>>,
    pub pomodoro_state: PomodoroStateInfo,
    pub window_state: WindowState,
    pub current_project: Option<String>,
    pub recent_tags: Vec<String>,
    pub notifications: Vec<NotificationState>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PomodoroStateInfo {
    pub state: PomodoroState,
    pub remaining_time: Duration,
    pub completed_count: u32,
    pub start_time: Option<DateTime<Local>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WindowState {
    pub position: Option<(i32, i32)>,
    pub size: Option<(u32, u32)>,
    pub is_minimized: bool,
    pub current_view: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationState {
    pub id: String,
    pub dismissed: bool,
    pub created_at: DateTime<Local>,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            is_modified: false,
            last_sync: None,
            pomodoro_state: PomodoroStateInfo {
                state: PomodoroState::Idle,
                remaining_time: Duration::from_secs(0),
                completed_count: 0,
                start_time: None,
            },
            window_state: WindowState {
                position: None,
                size: None,
                is_minimized: false,
                current_view: "overview".to_string(),
            },
            current_project: None,
            recent_tags: Vec::new(),
            notifications: Vec::new(),
        }
    }
}

pub struct AppStateManager {
    state: Arc<RwLock<AppState>>,
    config_path: std::path::PathBuf,
    auto_save: bool,
}

impl AppStateManager {
    pub fn new(config_dir: std::path::PathBuf, auto_save: bool) -> Result<Self> {
        std::fs::create_dir_all(&config_dir)?;
        let config_path = config_dir.join("app_state.json");
        
        let state = if config_path.exists() {
            let content = std::fs::read_to_string(&config_path)?;
            serde_json::from_str(&content)?
        } else {
            AppState::default()
        };

        Ok(Self {
            state: Arc::new(RwLock::new(state)),
            config_path,
            auto_save,
        })
    }

    pub fn get_state(&self) -> Result<AppState> {
        self.state
            .read()
            .map_err(|_| TimeTrackerError::Storage("Failed to read state".into()))
            .map(|state| state.clone())
    }

    pub fn update<F>(&self, f: F) -> Result<()>
    where
        F: FnOnce(&mut AppState),
    {
        let mut state = self.state
            .write()
            .map_err(|_| TimeTrackerError::Storage("Failed to write state".into()))?;
        
        f(&mut state);
        state.is_modified = true;

        if self.auto_save {
            self.save()?;
        }

        Ok(())
    }

    pub fn save(&self) -> Result<()> {
        let state = self.state
            .read()
            .map_err(|_| TimeTrackerError::Storage("Failed to read state".into()))?;

        if state.is_modified {
            let content = serde_json::to_string_pretty(&*state)?;
            std::fs::write(&self.config_path, content)?;
            
            // 重置修改标志
            if let Ok(mut state) = self.state.write() {
                state.is_modified = false;
            }
        }

        Ok(())
    }

    pub fn update_pomodoro_state(&self, 
        state: PomodoroState,
        remaining_time: Duration,
        completed_count: u32,
    ) -> Result<()> {
        self.update(|app_state| {
            app_state.pomodoro_state = PomodoroStateInfo {
                state,
                remaining_time,
                completed_count,
                start_time: Some(Local::now()),
            };
        })
    }

    pub fn update_window_state(&self,
        position: Option<(i32, i32)>,
        size: Option<(u32, u32)>,
        is_minimized: bool,
    ) -> Result<()> {
        self.update(|app_state| {
            app_state.window_state.position = position;
            app_state.window_state.size = size;
            app_state.window_state.is_minimized = is_minimized;
        })
    }

    pub fn add_notification(&self, id: String, notification: NotificationState) -> Result<()> {
        self.update(|app_state| {
            app_state.notifications.push(notification);
            // 保持最近的100条通知
            if app_state.notifications.len() > 100 {
                app_state.notifications.remove(0);
            }
        })
    }

    pub fn update_recent_tags(&self, tag: String) -> Result<()> {
        self.update(|app_state| {
            if !app_state.recent_tags.contains(&tag) {
                app_state.recent_tags.push(tag);
                // 保持最近的10个标签
                if app_state.recent_tags.len() > 10 {
                    app_state.recent_tags.remove(0);
                }
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_state_persistence() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let manager = AppStateManager::new(temp_dir.path().to_path_buf(), true)?;

        // 添加标签
        for i in 0..15 {
            manager.update_recent_tags(format!("tag_{}", i))?;
        }

        let state = manager.get_state()?;
        assert_eq!(state.recent_tags.len(), 10); // 验证限制
        assert_eq!(
            state.recent_tags.last().unwrap(),
            "tag_14"
        ); // 验证最新的在最后

        // 验证重复标签不会被添加
        manager.update_recent_tags("tag_14".to_string())?;
        let state = manager.get_state()?;
        assert_eq!(state.recent_tags.len(), 10);

        Ok(())
    }

    #[test]
    fn test_auto_save() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let config_path = temp_dir.path().join("app_state.json");

        // 创建自动保存的管理器
        let manager = AppStateManager::new(temp_dir.path().to_path_buf(), true)?;
        manager.update_window_state(Some((100, 100)), None, false)?;

        // 验证文件立即被创建并包含更新的状态
        assert!(config_path.exists());
        let content = std::fs::read_to_string(&config_path)?;
        let state: AppState = serde_json::from_str(&content)?;
        assert_eq!(state.window_state.position, Some((100, 100)));

        // 创建不自动保存的管理器
        let manager = AppStateManager::new(temp_dir.path().to_path_buf(), false)?;
        manager.update_window_state(Some((200, 200)), None, false)?;

        // 验证文件未被更新
        let content = std::fs::read_to_string(&config_path)?;
        let state: AppState = serde_json::from_str(&content)?;
        assert_eq!(state.window_state.position, Some((100, 100)));

        // 手动保存并验证
        manager.save()?;
        let content = std::fs::read_to_string(&config_path)?;
        let state: AppState = serde_json::from_str(&content)?;
        assert_eq!(state.window_state.position, Some((200, 200)));

        Ok(())
    }
}

        let manager = AppStateManager::new(temp_dir.path().to_path_buf(), true)?;

        // 更新状态
        manager.update_pomodoro_state(
            PomodoroState::Working,
            Duration::from_secs(1500),
            1,
        )?;

        manager.update_window_state(
            Some((100, 100)),
            Some((800, 600)),
            false,
        )?;

        // 创建新的管理器读取状态
        let manager2 = AppStateManager::new(temp_dir.path().to_path_buf(), true)?;
        let state = manager2.get_state()?;

        assert_eq!(state.pomodoro_state.state, PomodoroState::Working);
        assert_eq!(state.window_state.size, Some((800, 600)));

        Ok(())
    }

    #[test]
    fn test_notification_management() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let manager = AppStateManager::new(temp_dir.path().to_path_buf(), true)?;

        // 添加通知
        for i in 0..105 {
            manager.add_notification(
                format!("notification_{}", i),
                NotificationState {
                    id: format!("notification_{}", i),
                    dismissed: false,
                    created_at: Local::now(),
                },
            )?;
        }

        let state = manager.get_state()?;
        assert_eq!(state.notifications.len(), 100); // 验证限制
        assert_eq!(
            state.notifications.last().unwrap().id,
            "notification_104"
        ); // 验证最新的在最后

        Ok(())
    }

    #[test]
    fn test_recent_tags() -> Result<()> {
        let temp_dir = TempDir::new()?;