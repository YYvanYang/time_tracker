use crate::core::AppResult;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::time::Duration;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub general: GeneralConfig,
    pub pomodoro: PomodoroConfig,
    pub storage: StorageConfig,
    pub ui: UiConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneralConfig {
    pub autostart: bool,
    pub minimize_to_tray: bool,
    pub language: String,
    pub check_updates: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PomodoroConfig {
    pub work_duration: Duration,
    pub short_break_duration: Duration,
    pub long_break_duration: Duration,
    pub long_break_interval: u32,
    pub auto_start_breaks: bool,
    pub auto_start_pomodoros: bool,
    pub sound_enabled: bool,
    pub sound_volume: u8,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageConfig {
    pub data_dir: PathBuf,
    pub backup_enabled: bool,
    pub backup_interval: Duration,
    pub max_backup_count: u32,
    pub vacuum_threshold: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiConfig {
    pub theme: Theme,
    pub font_size: u32,
    pub window_width: u32,
    pub window_height: u32,
    pub show_seconds: bool,
    pub compact_mode: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Theme {
    Light,
    Dark,
    System,
}

impl Config {
    pub fn load() -> AppResult<Self> {
        let config_path = Self::get_config_path()?;
        if config_path.exists() {
            let contents = std::fs::read_to_string(&config_path)?;
            Ok(serde_json::from_str(&contents)?)
        } else {
            let config = Config::default();
            config.save()?;
            Ok(config)
        }
    }

    pub fn save(&self) -> AppResult<()> {
        let config_path = Self::get_config_path()?;
        if let Some(parent) = config_path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let contents = serde_json::to_string_pretty(self)?;
        std::fs::write(&config_path, contents)?;
        Ok(())
    }

    fn get_config_path() -> AppResult<PathBuf> {
        Ok(dirs::config_dir()
            .ok_or_else(|| crate::core::AppError::Config("Could not find config directory".into()))?
            .join("time_tracker")
            .join("config.json"))
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            general: GeneralConfig::default(),
            pomodoro: PomodoroConfig::default(),
            storage: StorageConfig::default(),
            ui: UiConfig::default(),
        }
    }
}

impl Default for GeneralConfig {
    fn default() -> Self {
        Self {
            autostart: false,
            minimize_to_tray: true,
            language: String::from("en"),
            check_updates: true,
        }
    }
}

impl Default for PomodoroConfig {
    fn default() -> Self {
        Self {
            work_duration: Duration::from_secs(25 * 60),
            short_break_duration: Duration::from_secs(5 * 60),
            long_break_duration: Duration::from_secs(15 * 60),
            long_break_interval: 4,
            auto_start_breaks: false,
            auto_start_pomodoros: false,
            sound_enabled: true,
            sound_volume: 80,
        }
    }
}

impl Default for StorageConfig {
    fn default() -> Self {
        Self {
            data_dir: dirs::data_dir()
                .unwrap_or_else(|| PathBuf::from("."))
                .join("time_tracker"),
            backup_enabled: true,
            backup_interval: Duration::from_secs(24 * 60 * 60),
            max_backup_count: 7,
            vacuum_threshold: 100 * 1024 * 1024,
        }
    }
}

impl Default for UiConfig {
    fn default() -> Self {
        Self {
            theme: Theme::System,
            font_size: 14,
            window_width: 800,
            window_height: 600,
            show_seconds: true,
            compact_mode: false,
        }
    }
} 