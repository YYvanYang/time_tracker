use crate::error::{Result, TimeTrackerError};
use serde::{Serialize, Deserialize};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppState {
    pub current_project: Option<String>,
    pub current_tags: Vec<String>,
    pub window_position: Option<(i32, i32)>,
    pub window_size: Option<(u32, u32)>,
    pub tasks: Vec<Task>,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            current_project: None,
            current_tags: Vec::new(),
            window_position: None,
            window_size: None,
            tasks: Vec::new(),
        }
    }
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
            tasks: Vec::new(),
        };

        Ok(Self {
            state: Arc::new(Mutex::new(state)),
            file_path: _data_dir.join("app_state.json"),
            auto_save: _auto_save,
        })
    }

    pub fn save_state(&mut self) -> Result<()> {
        let state = self.state.lock().unwrap();
        
        // 将状态序列化为 JSON
        let state_json = serde_json::to_string(&*state)?;
        
        // 确保目录存在
        if let Some(parent) = self.file_path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        
        // 写入文件
        std::fs::write(&self.file_path, state_json)?;
        
        Ok(())
    }

    pub fn get_state(&self) -> Result<std::sync::MutexGuard<AppState>> {
        self.state.lock().map_err(|e| {
            TimeTrackerError::State(format!("Failed to lock app state: {}", e))
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    pub id: u32,
    pub title: String,
    pub completed: bool,
    pub created_at: chrono::DateTime<chrono::Local>,
}

impl Task {
    pub fn new(title: String) -> Self {
        use rand::Rng;
        Self {
            id: rand::thread_rng().gen(),
            title,
            completed: false,
            created_at: chrono::Local::now(),
        }
    }
}