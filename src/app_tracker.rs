use crate::error::{Result, TimeTrackerError};
use crate::platform::{PlatformOperations, WindowInfo as PlatformWindowInfo};
use crate::storage::AppUsageRecord;
use chrono::{DateTime, Local};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppUsageData {
    pub app_name: String,
    pub window_title: String,
    pub start_time: DateTime<Local>,
    pub duration: Duration,
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
    pub tracking_interval: Duration,
    pub idle_threshold: Duration,
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
            tracking_interval: Duration::from_secs(30),
            idle_threshold: Duration::from_secs(300),
            category_rules,
        }
    }
}

#[allow(dead_code)]
pub struct AppTracker {
    config: AppUsageConfig,
    current_app: Arc<Mutex<Option<AppUsageData>>>,
    last_check: Arc<Mutex<Instant>>,
    storage: Arc<Mutex<Vec<AppUsageData>>>,
    last_active: Arc<Mutex<Instant>>,
    platform: Arc<Mutex<Option<Box<dyn PlatformOperations + Send>>>>,
}

impl AppTracker {
    pub fn new(config: AppUsageConfig) -> Result<Self> {
        let platform = crate::platform::init()?;
        Ok(Self {
            config,
            current_app: Arc::new(Mutex::new(None)),
            last_check: Arc::new(Mutex::new(Instant::now())),
            storage: Arc::new(Mutex::new(Vec::new())),
            last_active: Arc::new(Mutex::new(Instant::now())),
            platform: Arc::new(Mutex::new(Some(Box::new(platform)))),
        })
    }

    pub fn update(&mut self) -> Result<()> {
        let now = Instant::now();
        
        let window_info = {
            let platform_guard = self.platform.lock()?;
            platform_guard.as_ref()
                .ok_or_else(|| TimeTrackerError::Platform("Platform not initialized".into()))?
                .get_active_window()?
        };

        self.switch_to_new_app(window_info)?;

        *self.last_check.lock()? = now;
        Ok(())
    }

    fn handle_idle_period(&mut self) -> Result<()> {
        if let Some(current_app) = self.current_app.lock()?.take() {
            // 保存当前应用使用记录
            self.storage.lock()
                .map_err(|_| TimeTrackerError::Platform("Failed to lock storage".into()))?
                .push(current_app);
        }
        Ok(())
    }

    fn update_current_app_duration(&mut self) -> Result<()> {
        if let Some(app) = self.current_app.lock()?.as_mut() {
            let elapsed = self.last_check.lock()?.elapsed();
            app.duration += elapsed;
        }
        Ok(())
    }

    pub fn switch_to_new_app(&mut self, window_info: PlatformWindowInfo) -> Result<()> {
        let app_name = window_info.app_name.clone();
        
        let app_info = AppUsageData {
            app_name: window_info.app_name,
            window_title: window_info.window_title,
            start_time: chrono::Local::now(),
            duration: Duration::default(),
            is_productive: self.is_productive(&app_name),
            category: self.get_app_category(&app_name),
        };

        // 获取并更新当前应用
        let mut current_app = self.current_app.lock()?;
        if let Some(app) = current_app.as_mut() {
            // 更新结束时间和持续时间
            app.duration = self.last_check.lock()?.elapsed();
            // 保存当前应用记录
            self.storage.lock()?.push(app.clone());
        }

        // 设置新的当前应用
        *current_app = Some(app_info);
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

    pub fn get_usage_stats(&self, since: DateTime<Local>) -> Result<AppUsageStats> {
        let mut stats = AppUsageStats {
            total_time: Duration::from_secs(0),
            productive_time: Duration::from_secs(0),
            unproductive_time: Duration::from_secs(0),
            most_used_app: None,
            app_usage: HashMap::new(),
            category_usage: HashMap::new(),
        };

        let storage = self.storage.lock()?;
        for app in storage.iter().filter(|app| app.start_time >= since) {
            stats.total_time += app.duration;
            if app.is_productive {
                stats.productive_time += app.duration;
            } else {
                stats.unproductive_time += app.duration;
            }

            // 更新应用使用统计
            let entry = stats.app_usage.entry(app.app_name.clone())
                .or_insert_with(|| Duration::from_secs(0));
            *entry += app.duration;

            // 更新类别使用统计
            let entry = stats.category_usage.entry(app.category.clone())
                .or_insert_with(|| Duration::from_secs(0));
            *entry += app.duration;
        }

        // 找出使用时间最长的应用
        if let Some((app, _)) = stats.app_usage.iter()
            .max_by_key(|(_, &duration)| duration) {
            stats.most_used_app = Some(app.clone());
        }

        Ok(stats)
    }

    pub fn export_data(&self) -> Result<Vec<AppUsageData>> {
        Ok(self.storage.lock()
            .map_err(|_| TimeTrackerError::Platform("Failed to lock storage".into()))?
            .clone())
    }

    fn validate_app_data(&self, data: &AppUsageData) -> Result<()> {
        if data.duration.as_secs() == 0 {
            return Err(TimeTrackerError::Platform("Invalid duration".into()));
        }
        if data.app_name.is_empty() {
            return Err(TimeTrackerError::Platform("Empty app name".into()));
        }
        Ok(())
    }

    pub fn record_window(&mut self, window_info: PlatformWindowInfo) -> Result<()> {
        let _record = AppUsageRecord {
            id: None,
            app_name: window_info.app_name.clone(),
            window_title: window_info.title,
            start_time: Local::now(),
            duration: Duration::from_secs(0),
            category: String::new(),
            is_productive: self.is_productive(&window_info.app_name),
        };
        Ok(())
    }

    fn check_active_window(&mut self) -> Result<()> {
        let window_info = {
            let platform_guard = self.platform.lock()?;
            if let Some(platform) = platform_guard.as_ref() {
                platform.get_active_window()?
            } else {
                return Err(TimeTrackerError::Platform("Platform not initialized".into()));
            }
        };

        self.switch_to_new_app(window_info)?;
        Ok(())
    }

    // 添加保存应用记录的方法
    fn save_app_record(&mut self, app_data: AppUsageData) -> Result<()> {
        self.storage.lock()?.push(app_data);
        Ok(())
    }
}

#[derive(Debug)]
pub struct AppUsageStats {
    pub total_time: Duration,
    pub productive_time: Duration,
    pub unproductive_time: Duration,
    pub most_used_app: Option<String>,
    pub app_usage: HashMap<String, Duration>,
    pub category_usage: HashMap<AppCategory, Duration>,
}

impl AppUsageStats {
    pub fn new() -> Self {
        Self {
            total_time: Duration::from_secs(0),
            productive_time: Duration::from_secs(0),
            unproductive_time: Duration::from_secs(0),
            most_used_app: None,
            app_usage: HashMap::new(),
            category_usage: HashMap::new(),
        }
    }

    pub fn get_productivity_ratio(&self) -> f64 {
        if self.total_time.as_secs() == 0 {
            0.0
        } else {
            self.productive_time.as_secs_f64() / self.total_time.as_secs_f64()
        }
    }

    pub fn get_most_used_category(&self) -> Option<(&AppCategory, Duration)> {
        self.category_usage
            .iter()
            .max_by_key(|&(_, duration)| duration)
            .map(|(category, duration)| (category, *duration))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn test_app_category_detection() {
        let config = AppUsageConfig::default();
        let tracker = AppTracker::new(config).expect("Failed to create AppTracker");

        assert_eq!(tracker.get_app_category("Visual Studio Code"), AppCategory::Development);
        assert_eq!(tracker.get_app_category("Microsoft Excel"), AppCategory::Productivity);
        assert_eq!(tracker.get_app_category("Unknown App"), AppCategory::Other);
    }

    #[test]
    fn test_productivity_detection() {
        let config = AppUsageConfig::default();
        let tracker = AppTracker::new(config).expect("Failed to create AppTracker");

        assert!(tracker.is_productive("Visual Studio Code"));
        assert!(!tracker.is_productive("YouTube"));
    }

    #[test]
    fn test_usage_stats() -> Result<()> {
        let config = AppUsageConfig::default();
        let mut tracker = AppTracker::new(config).expect("Failed to create AppTracker");

        // 模拟应用使用
        let window_info = crate::platform::WindowInfo {
            app_name: "Visual Studio Code".to_string(),
            title: "main.rs".to_string(),
            window_title: "main.rs".to_string(),
            process_name: "code.exe".to_string(),
            process_id: 1234,
        };

        tracker.switch_to_new_app(window_info)?;
        std::thread::sleep(Duration::from_secs(1));
        tracker.update()?;

        let stats = tracker.get_usage_stats(Local::now() - chrono::Duration::hours(1))?;
        assert!(stats.productive_time > Duration::from_secs(0));
        assert_eq!(stats.most_used_app, Some("Visual Studio Code".to_string()));

        Ok(())
    }
}