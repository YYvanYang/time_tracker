use crate::error::Result;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppState {
    pub window_position: Option<(i32, i32)>,
    pub window_size: Option<(u32, u32)>,
    pub last_view: String,
}

pub struct AppStateManager {
    state: AppState,
    file_path: PathBuf,
    auto_save: bool,
}

impl AppStateManager {
    pub fn new(data_dir: PathBuf, auto_save: bool) -> Result<Self> {
        let file_path = data_dir.join("app_state.json");
        let state = if file_path.exists() {
            let contents = std::fs::read_to_string(&file_path)?;
            serde_json::from_str(&contents)?
        } else {
            AppState {
                window_position: None,
                window_size: None,
                last_view: "overview".to_string(),
            }
        };

        Ok(Self {
            state,
            file_path,
            auto_save,
        })
    }
}

#[derive(Debug, Clone)]
pub struct Project {
    pub id: i64,
    pub name: String,
    pub description: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
} 