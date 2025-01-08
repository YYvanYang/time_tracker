use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub data_dir: PathBuf,
    pub autostart: bool,
    pub minimize_to_tray: bool,
    pub start_minimized: bool,
    pub theme: String,
    pub language: String,
    pub pomodoro_duration: i32,
    pub short_break_duration: i32,
    pub long_break_duration: i32,
    pub long_break_interval: i32,
    pub auto_start_break: bool,
    pub auto_start_work: bool,
    pub notification_sound: bool,
    pub idle_detection_enabled: bool,
    pub idle_detection_threshold: i32,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            data_dir: dirs::data_dir()
                .unwrap_or_else(|| PathBuf::from("./data"))
                .join("time_tracker"),
            autostart: false,
            minimize_to_tray: true,
            start_minimized: false,
            theme: "system".to_string(),
            language: "en".to_string(),
            pomodoro_duration: 25,
            short_break_duration: 5,
            long_break_duration: 15,
            long_break_interval: 4,
            auto_start_break: true,
            auto_start_work: false,
            notification_sound: true,
            idle_detection_enabled: true,
            idle_detection_threshold: 300,
        }
    }
}

pub struct ConfigManager {
    config_path: PathBuf,
}

impl ConfigManager {
    pub fn new() -> Self {
        let config_dir = dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("./config"))
            .join("time_tracker");
        
        std::fs::create_dir_all(&config_dir).unwrap_or_default();
        
        Self {
            config_path: config_dir.join("config.json"),
        }
    }

    pub fn load(&self) -> Config {
        if self.config_path.exists() {
            if let Ok(contents) = std::fs::read_to_string(&self.config_path) {
                if let Ok(config) = serde_json::from_str(&contents) {
                    return config;
                }
            }
        }
        Config::default()
    }

    pub fn save(&self, config: &Config) -> std::io::Result<()> {
        let contents = serde_json::to_string_pretty(config)?;
        std::fs::write(&self.config_path, contents)
    }
} 