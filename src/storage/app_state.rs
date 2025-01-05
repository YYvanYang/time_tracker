// src/storage/app_state.rs

use crate::error::Result;
use serde::{Serialize, Deserialize};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppState {
    pub current_project: Option<String>,
    pub current_tags: Vec<String>,
    pub window_position: Option<(i32, i32)>,
    pub window_size: Option<(u32, u32)>,
}

pub struct AppStateManager {
    state: Arc<Mutex<AppState>>,
    file_path: PathBuf,
    auto_save: bool,
}

impl AppStateManager {
    pub fn new(_data_dir: PathBuf, _auto_save: bool) -> Result<Self> {
        let state = AppState {
            current_project: None,
            current_tags: Vec::new(),
            window_position: None,
            window_size: None,
        };

        Ok(Self {
            state: Arc::new(Mutex::new(state)),
            file_path: _data_dir.join("app_state.json"),
            auto_save: _auto_save,
        })
    }

    pub fn save_state(&mut self) -> Result<()> {
        let state = self.state.lock().unwrap();
        // TODO: 实现保存状态的逻辑
        Ok(())
    }

    pub fn get_state(&self) -> Result<std::sync::MutexGuard<AppState>> {
        self.state.lock().map_err(|e| {
            TimeTrackerError::State(format!("Failed to lock app state: {}", e))
        })
    }
}