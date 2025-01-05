use crate::error::{Result, TimeTrackerError};
use crate::platform::PlatformOperations;
use chrono::{DateTime, Duration, Local};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Instant, SystemTime};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppUsageData {
    pub app_name: String,
    pub window_title: String,
    pub start_time: DateTime<Local>,
    pub duration: std::time::Duration,
    pub is_productive: bool,
    pub category: AppCategory,
}

#[derive(Debug, Clone, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub enum AppCategory {
    Development,
    Communication,
    Productivity,
    Entertainment,
    Other,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppUsageConfig {
    pub productive_apps: Vec<String>,
    pub unproductive_apps: Vec<String>,
    pub tracking_interval: std::time::Duration,
    pub idle_threshold: std::time::Duration,
    pub category_rules: HashMap<String, AppCategory>,
}

impl Default for AppUsageConfig {
    fn default() -> Self {
        let mut category_rules = HashMap::new();
        category_rules.insert("code".to_string(), AppCategory::Development);
        category_rules.insert("visual studio".to_string(), AppCategory::Development);
        category_rules.insert("chrome".to_string(), AppCategory::Productivity);
        category_rules.insert("firefox".to_string(), AppCategory::Productivity);
        category_rules.insert("word".to_string(), AppCategory::Productivity);
        category_rules.insert("excel".to_string(), AppCategory::Productivity);
        category_rules.insert("slack".to_string(), AppCategory::Communication);
        category_rules.insert("teams".to_string(), AppCategory::Communication);

        Self {
            productive_apps: vec![
                "code".to_string(),
                "visual studio".to_string(),
                "word".to_string(),
                "excel".to_string(),
            ],
            unproductive_apps: vec![
                "game".to_string(),
                "youtube".to_string(),
            ],
            tracking_interval: std::time::Duration::from_secs(30),
            idle_threshold: std::time::Duration::from_secs(300),
            category_rules,
        }
    }
}

pub struct AppTracker {
    config: AppUsageConfig,
    current_app: Option<AppUsageData>,
    last_check: Instant,
    storage: Arc<Mutex<Vec<AppUsageData>>>,
    last_active: Instant,
    platform: Option<Box<dyn PlatformOperations>>,
}

impl AppTracker {
    pub fn new(config: AppUsageConfig) -> Self {
        Self {
            config,
            current_app: None,
            last_check: Instant::now(),
            storage: Arc::new(Mutex::new(Vec::new())),
            last_active: Instant::now(),
            platform: None,
        }
    }

    pub fn update(&mut self) -> Result<()> {
        let now = Instant::now();
        if now.duration_since(self.last_check) < self.config.tracking_interval {
            return Ok(());
        }

        if let Some(platform) = &self.platform {
            if let Ok(window_info) = platform.get_active_window() {
                self.switch_to_new_app(window_info)?;
            }
        }

        Ok(())
    }

    fn handle_idle_period(&mut self) -> Result<()> {
        if let Some(current_app) = self.current_app.take() {
            // 保存当前应用使用记录
            self.storage.lock()
                .map_err(|_| TimeTrackerError::Platform("Failed to lock storage".into()))?
                .push(current_app);
        }
        Ok(())
    }

    fn update_current_app_duration(&mut self) -> Result<()> {
        if let Some(app) = &mut self.current_app {
            let elapsed = self.last_check.elapsed();
            app.duration += elapsed;
        }
        Ok(())
    }

    fn switch_to_new_app(&mut self, mut window_info: crate::platform::WindowInfo) -> Result<()> {
        // 保存之前的应用记录
        if let Some(current_app) = self.current_app.take() {
            self.storage.lock()
                .map_err(|_| TimeTrackerError::Platform("Failed to lock storage".into()))?
                .push(current_app);
        }

        let app_name = window_info.app_name.clone();
        let is_productive = self.is_productive(&app_name);
        let category = self.get_app_category(&app_name);

        // 创建新的应用记录
        self.current_app = Some(AppUsageData {
            app_name,
            window_title: window_info.window_title,
            start_time: Local::now(),
            duration: std::time::Duration::from_secs(0),
            is_productive,
            category,
        });

        self.last_active = Instant::now();
        Ok(())
    }

    fn is_productive(&self, app_name: &str) -> bool {
        let app_name_lower = app_name.to_lowercase();
        self.config.productive_apps.iter()
            .any(|productive| app_name_lower.contains(&productive.to_lowercase()))
            && !self.config.unproductive_apps.iter()
                .any(|unproductive| app_name_lower.contains(&unproductive.to_lowercase()))
    }

    fn get_app_category(&self, app_name: &str) -> AppCategory {
        let app_name_lower = app_name.to_lowercase();
        for (pattern, category) in &self.config.category_rules {
            if app_name_lower.contains(&pattern.to_lowercase()) {
                return category.clone();
            }
        }
        AppCategory::Other
    }

    pub fn get_usage_stats(&self, start_time: DateTime<Local>) -> Result<AppUsageStats> {
        let storage = self.storage.lock()
            .map_err(|_| TimeTrackerError::Platform("Failed to lock storage".into()))?;

        let mut stats = AppUsageStats::new();
        for usage in storage.iter().filter(|u| u.start_time >= start_time) {
            stats.add_usage(usage);
        }

        Ok(stats)
    }

    pub fn export_data(&self) -> Result<Vec<AppUsageData>> {
        Ok(self.storage.lock()
            .map_err(|_| TimeTrackerError::Platform("Failed to lock storage".into()))?
            .clone())
    }
}

#[derive(Debug)]
pub struct AppUsageStats {
    pub total_time: std::time::Duration,
    pub productive_time: std::time::Duration,
    pub app_durations: HashMap<String, std::time::Duration>,
    pub category_durations: HashMap<AppCategory, std::time::Duration>,
    pub most_used_app: Option<String>,
}

impl AppUsageStats {
    fn new() -> Self {
        Self {
            total_time: std::time::Duration::from_secs(0),
            productive_time: std::time::Duration::from_secs(0),
            app_durations: HashMap::new(),
            category_durations: HashMap::new(),
            most_used_app: None,
        }
    }

    fn add_usage(&mut self, usage: &AppUsageData) {
        self.total_time += usage.duration;
        if usage.is_productive {
            self.productive_time += usage.duration;
        }

        // 更新应用时长
        *self.app_durations
            .entry(usage.app_name.clone())
            .or_insert(std::time::Duration::from_secs(0)) += usage.duration;

        // 更新类别时长
        *self.category_durations
            .entry(usage.category.clone())
            .or_insert(std::time::Duration::from_secs(0)) += usage.duration;

        // 更新最常用应用
        if let Some(current_most_used) = &self.most_used_app {
            if self.app_durations[&usage.app_name] > self.app_durations[current_most_used] {
                self.most_used_app = Some(usage.app_name.clone());
            }
        } else {
            self.most_used_app = Some(usage.app_name.clone());
        }
    }

    pub fn get_productivity_percentage(&self) -> f64 {
        if self.total_time.as_secs() == 0 {
            0.0
        } else {
            self.productive_time.as_secs_f64() / self.total_time.as_secs_f64() * 100.0
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn test_app_category_detection() {
        let config = AppUsageConfig::default();
        let tracker = AppTracker::new(config);

        assert_eq!(tracker.get_app_category("Visual Studio Code"), AppCategory::Development);
        assert_eq!(tracker.get_app_category("Microsoft Excel"), AppCategory::Productivity);
        assert_eq!(tracker.get_app_category("Unknown App"), AppCategory::Other);
    }

    #[test]
    fn test_productivity_detection() {
        let config = AppUsageConfig::default();
        let tracker = AppTracker::new(config);

        assert!(tracker.is_productive("Visual Studio Code"));
        assert!(!tracker.is_productive("YouTube"));
    }

    #[test]
    fn test_usage_stats() -> Result<()> {
        let config = AppUsageConfig::default();
        let mut tracker = AppTracker::new(config);

        // 模拟应用使用
        let window_info = crate::platform::WindowInfo {
            app_name: "Visual Studio Code".to_string(),
            window_title: "main.rs".to_string(),
            process_id: 1234,
        };

        tracker.switch_to_new_app(window_info)?;
        std::thread::sleep(Duration::from_secs(1));
        tracker.update()?;

        let stats = tracker.get_usage_stats(Local::now() - Duration::hours(1))?;
        assert!(stats.productive_time > Duration::from_secs(0));
        assert_eq!(stats.most_used_app, Some("Visual Studio Code".to_string()));

        Ok(())
    }
}