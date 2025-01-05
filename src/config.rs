use crate::error::{Result, TimeTrackerError};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::fs;
use std::time::Duration;
use notify::{Watcher, RecursiveMode, watcher};
use std::sync::mpsc::channel;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    #[serde(default)]
    pub general: GeneralConfig,
    
    #[serde(default)]
    pub pomodoro: PomodoroConfig,
    
    #[serde(default)]
    pub shutdown: ShutdownConfig,
    
    #[serde(default)]
    pub ui: UiConfig,
    
    #[serde(default)]
    pub storage: StorageConfig,
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
pub struct ShutdownConfig {
    pub enabled: bool,
    pub pomodoros_before_shutdown: u32,
    pub delay_minutes: u32,
    pub show_confirmation: bool,
    pub auto_save_reminder: bool,
    pub notification_minutes: Vec<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scheduled_time: Option<chrono::NaiveTime>,
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageConfig {
    pub data_dir: PathBuf,
    pub backup_enabled: bool,
    pub backup_interval: Duration,
    pub keep_data_days: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Theme {
    Light,
    Dark,
    System,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            general: GeneralConfig::default(),
            pomodoro: PomodoroConfig::default(),
            shutdown: ShutdownConfig::default(),
            ui: UiConfig::default(),
            storage: StorageConfig::default(),
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

impl Default for ShutdownConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            pomodoros_before_shutdown: 4,
            delay_minutes: 30,
            show_confirmation: true,
            auto_save_reminder: true,
            notification_minutes: vec![30, 15, 5, 1],
            scheduled_time: None,
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

impl Default for StorageConfig {
    fn default() -> Self {
        Self {
            data_dir: dirs::data_dir()
                .unwrap_or_else(|| PathBuf::from("."))
                .join("time_tracker"),
            backup_enabled: true,
            backup_interval: Duration::from_secs(24 * 60 * 60), // 每天
            keep_data_days: 90,
        }
    }
}

impl Config {
    pub fn load() -> Result<Self> {
        let config_path = Self::get_config_path()?;
        if config_path.exists() {
            let contents = fs::read_to_string(&config_path)?;
            let config: Config = serde_json::from_str(&contents)?;
            config.validate()?;
            Ok(config)
        } else {
            let config = Config::default();
            config.save()?;
            Ok(config)
        }
    }

    pub fn save(&self) -> Result<()> {
        self.validate()?;
        let config_path = Self::get_config_path()?;
        if let Some(parent) = config_path.parent() {
            fs::create_dir_all(parent)?;
        }
        let contents = serde_json::to_string_pretty(self)?;
        fs::write(&config_path, contents)?;
        Ok(())
    }

    fn get_config_path() -> Result<PathBuf> {
        dirs::config_dir()
            .map(|p| p.join("time_tracker").join("config.json"))
            .ok_or_else(|| TimeTrackerError::Config(
                "Could not determine config directory".into()
            ))
    }

    pub fn validate(&self) -> Result<()> {
        // 验证番茄钟配置
        if self.pomodoro.work_duration < Duration::from_secs(60) {
            return Err(TimeTrackerError::Config(
                "Work duration must be at least 1 minute".into()
            ));
        }
        if self.pomodoro.short_break_duration < Duration::from_secs(30) {
            return Err(TimeTrackerError::Config(
                "Short break duration must be at least 30 seconds".into()
            ));
        }
        if self.pomodoro.long_break_duration < Duration::from_secs(60) {
            return Err(TimeTrackerError::Config(
                "Long break duration must be at least 1 minute".into()
            ));
        }
        if self.pomodoro.long_break_interval < 1 {
            return Err(TimeTrackerError::Config(
                "Long break interval must be at least 1".into()
            ));
        }

        // 验证关机配置
        if self.shutdown.enabled {
            if self.shutdown.delay_minutes == 0 {
                return Err(TimeTrackerError::Config(
                    "Shutdown delay must be greater than 0".into()
                ));
            }
            if self.shutdown.pomodoros_before_shutdown == 0 {
                return Err(TimeTrackerError::Config(
                    "Pomodoros before shutdown must be greater than 0".into()
                ));
            }
        }

        // 验证UI配置
        if self.ui.window_width < 400 || self.ui.window_height < 300 {
            return Err(TimeTrackerError::Config(
                "Window size too small".into()
            ));
        }
        if self.ui.font_size < 8 || self.ui.font_size > 32 {
            return Err(TimeTrackerError::Config(
                "Font size must be between 8 and 32".into()
            ));
        }

        // 验证存储配置
        if self.storage.keep_data_days == 0 {
            return Err(TimeTrackerError::Config(
                "Keep data days must be greater than 0".into()
            ));
        }

        Ok(())
    }

    pub fn watch_changes(&self) -> Result<()> {
        let config_path = Self::get_config_path()?;
        let (tx, rx) = channel();

        let mut watcher = watcher(tx, Duration::from_secs(2))?;
        watcher.watch(config_path, RecursiveMode::NonRecursive)?;

        std::thread::spawn(move || {
            for result in rx {
                match result {
                    Ok(event) => {
                        if let Err(e) = Self::reload() {
                            log::error!("Failed to reload config: {}", e);
                        }
                    }
                    Err(e) => log::error!("Watch error: {}", e),
                }
            }
        });

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;
    use tempfile::TempDir;

    #[test]
    fn test_config_validation() {
        let mut config = Config::default();
        assert!(config.validate().is_ok());

        // Test invalid work duration
        config.pomodoro.work_duration = Duration::from_secs(30);
        assert!(config.validate().is_err());

        // Reset and test invalid window size
        config = Config::default();
        config.ui.window_width = 300;
        assert!(config.validate().is_err());

        // Test invalid font size
        config = Config::default();
        config.ui.font_size = 6;
        assert!(config.validate().is_err());

        // Test invalid shutdown settings
        config = Config::default();
        config.shutdown.enabled = true;
        config.shutdown.delay_minutes = 0;
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_config_save_load() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let config_path = temp_dir.path().join("config.json");
        
        // 创建测试配置
        let mut config = Config::default();
        config.general.language = "zh-CN".to_string();
        config.pomodoro.work_duration = Duration::from_secs(30 * 60);

        // 保存配置
        std::fs::create_dir_all(config_path.parent().unwrap())?;
        let contents = serde_json::to_string_pretty(&config)?;
        std::fs::write(&config_path, contents)?;

        // 加载并验证配置
        let loaded_config: Config = serde_json::from_str(&std::fs::read_to_string(&config_path)?)?;
        assert_eq!(loaded_config.general.language, "zh-CN");
        assert_eq!(loaded_config.pomodoro.work_duration, Duration::from_secs(30 * 60));

        Ok(())
    }

    #[test]
    fn test_default_values() {
        let config = Config::default();
        
        // 验证默认值
        assert_eq!(config.general.language, "en");
        assert_eq!(config.pomodoro.work_duration, Duration::from_secs(25 * 60));
        assert_eq!(config.ui.theme, Theme::System);
        assert!(config.storage.backup_enabled);
        assert!(!config.shutdown.enabled);
    }
}