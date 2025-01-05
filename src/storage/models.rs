// src/storage/models.rs

use chrono::{DateTime, Local};
use serde::{Serialize, Deserialize};
use std::time::Duration;
use egui::Color32;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppUsageRecord {
    pub id: Option<i64>,
    pub app_name: String,
    pub window_title: String,
    pub start_time: DateTime<Local>,
    pub duration: Duration,
    pub category: String,
    pub is_productive: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PomodoroRecord {
    pub id: Option<i64>,
    pub start_time: DateTime<Local>,
    pub end_time: DateTime<Local>,
    pub status: PomodoroStatus,
    pub notes: Option<String>,
    pub tags: Vec<String>,
    pub project_id: Option<i64>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum PomodoroStatus {
    Completed,
    Interrupted,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Project {
    pub id: Option<i64>,
    pub name: String,
    pub description: Option<String>,
    pub color: Option<String>,
    pub created_at: DateTime<Local>,
    pub updated_at: DateTime<Local>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tag {
    pub id: Option<i64>,
    pub name: String,
    pub color: Option<String>,
    pub created_at: DateTime<Local>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DailySummary {
    pub id: Option<i64>,
    pub date: DateTime<Local>,
    pub total_work_time: Duration,
    pub productive_time: Duration,
    pub completed_pomodoros: u32,
    pub interrupted_pomodoros: u32,
    pub most_used_app: Option<String>,
}

impl AppUsageRecord {
    pub fn new(
        app_name: String,
        window_title: String,
        start_time: DateTime<Local>,
        duration: Duration,
        category: String,
        is_productive: bool,
    ) -> Self {
        Self {
            id: None,
            app_name,
            window_title,
            start_time,
            duration,
            category,
            is_productive,
        }
    }
}

impl PomodoroRecord {
    pub fn new(
        start_time: DateTime<Local>,
        end_time: DateTime<Local>,
        status: PomodoroStatus,
        notes: Option<String>,
        tags: Vec<String>,
        project_id: Option<i64>,
    ) -> Self {
        Self {
            id: None,
            start_time,
            end_time,
            status,
            notes,
            tags,
            project_id,
        }
    }

    pub fn duration(&self) -> Duration {
        self.end_time
            .signed_duration_since(self.start_time)
            .to_std()
            .unwrap_or(Duration::from_secs(0))
    }
}

impl Project {
    pub fn new(name: String, description: Option<String>, color: Option<String>) -> Self {
        let now = Local::now();
        Self {
            id: None,
            name,
            description,
            color,
            created_at: now,
            updated_at: now,
        }
    }
}

impl Tag {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            id: None,
            name: name.into(),
            color: None,
            created_at: Local::now(),
        }
    }

    pub fn with_color<S: Into<String>>(mut self, color: S) -> Self {
        self.color = Some(color.into());
        self
    }

    pub fn show(&self, ui: &mut egui::Ui) {
        let color = self.color.as_ref()
            .and_then(|hex| Color32::from_rgb_hex(hex).ok())
            .unwrap_or(Color32::WHITE);
        ui.colored_label(color, &self.name);
    }
}

impl DailySummary {
    pub fn new(date: DateTime<Local>) -> Self {
        Self {
            id: None,
            date,
            total_work_time: Duration::from_secs(0),
            productive_time: Duration::from_secs(0),
            completed_pomodoros: 0,
            interrupted_pomodoros: 0,
            most_used_app: None,
        }
    }

    pub fn productivity_ratio(&self) -> f64 {
        if self.total_work_time.as_secs() == 0 {
            0.0
        } else {
            self.productive_time.as_secs_f64() / self.total_work_time.as_secs_f64()
        }
    }
}

// 数据库查询相关的特征实现
pub trait DatabaseRecord {
    fn table_name() -> &'static str;
    fn create_table_sql() -> &'static str;
}

impl DatabaseRecord for AppUsageRecord {
    fn table_name() -> &'static str {
        "app_usage"
    }

    fn create_table_sql() -> &'static str {
        "CREATE TABLE IF NOT EXISTS app_usage (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            app_name TEXT NOT NULL,
            window_title TEXT NOT NULL,
            start_time DATETIME NOT NULL,
            duration INTEGER NOT NULL,
            category TEXT NOT NULL,
            is_productive BOOLEAN NOT NULL,
            created_at DATETIME DEFAULT CURRENT_TIMESTAMP
        )"
    }
}

impl DatabaseRecord for PomodoroRecord {
    fn table_name() -> &'static str {
        "pomodoro_records"
    }

    fn create_table_sql() -> &'static str {
        "CREATE TABLE IF NOT EXISTS pomodoro_records (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            start_time DATETIME NOT NULL,
            end_time DATETIME NOT NULL,
            status TEXT NOT NULL,
            notes TEXT,
            project_id INTEGER,
            created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
            FOREIGN KEY(project_id) REFERENCES projects(id) ON DELETE SET NULL
        )"
    }
}

impl DatabaseRecord for Project {
    fn table_name() -> &'static str {
        "projects"
    }

    fn create_table_sql() -> &'static str {
        "CREATE TABLE IF NOT EXISTS projects (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            name TEXT NOT NULL UNIQUE,
            description TEXT,
            color TEXT,
            created_at DATETIME NOT NULL,
            updated_at DATETIME NOT NULL
        )"
    }
}

impl DatabaseRecord for Tag {
    fn table_name() -> &'static str {
        "tags"
    }

    fn create_table_sql() -> &'static str {
        "CREATE TABLE IF NOT EXISTS tags (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            name TEXT NOT NULL UNIQUE,
            color TEXT,
            created_at DATETIME NOT NULL
        )"
    }
}

impl DatabaseRecord for DailySummary {
    fn table_name() -> &'static str {
        "daily_summaries"
    }

    fn create_table_sql() -> &'static str {
        "CREATE TABLE IF NOT EXISTS daily_summaries (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            date DATE NOT NULL UNIQUE,
            total_work_time INTEGER NOT NULL,
            productive_time INTEGER NOT NULL,
            completed_pomodoros INTEGER NOT NULL,
            interrupted_pomodoros INTEGER NOT NULL,
            most_used_app TEXT,
            created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
            updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
        )"
    }
}

// 用于数据库关联表的模型
#[derive(Debug, Clone)]
pub struct PomodoroTag {
    pub pomodoro_id: i64,
    pub tag_id: i64,
}

impl DatabaseRecord for PomodoroTag {
    fn table_name() -> &'static str {
        "pomodoro_tags"
    }

    fn create_table_sql() -> &'static str {
        "CREATE TABLE IF NOT EXISTS pomodoro_tags (
            pomodoro_id INTEGER NOT NULL,
            tag_id INTEGER NOT NULL,
            created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
            PRIMARY KEY (pomodoro_id, tag_id),
            FOREIGN KEY(pomodoro_id) REFERENCES pomodoro_records(id) ON DELETE CASCADE,
            FOREIGN KEY(tag_id) REFERENCES tags(id) ON DELETE CASCADE
        )"
    }
}

// 值对象 - 用于统计和分析
#[derive(Debug, Clone)]
pub struct TimeDistribution {
    pub hour: u32,
    pub weekday: u32,
    pub duration: Duration,
    pub productivity: f64,
}

#[derive(Debug, Clone)]
pub struct CategoryStats {
    pub category: String,
    pub total_time: Duration,
    pub percentage: f64,
}

#[derive(Debug, Clone)]
pub struct ProductivityStats {
    pub total_time: Duration,
    pub productive_time: Duration,
    pub productivity_ratio: f64,
    pub most_productive_hour: Option<u32>,
    pub most_productive_day: Option<u32>,
}

// 添加颜色转换辅助函数
trait ColorExt {
    fn from_rgb_hex(hex: &str) -> Result<Color32, &'static str>;
}

impl ColorExt for Color32 {
    fn from_rgb_hex(hex: &str) -> Result<Color32, &'static str> {
        let hex = hex.trim_start_matches('#');
        if hex.len() != 6 {
            return Err("Invalid hex color length");
        }
        
        let r = u8::from_str_radix(&hex[0..2], 16).map_err(|_| "Invalid red component")?;
        let g = u8::from_str_radix(&hex[2..4], 16).map_err(|_| "Invalid green component")?;
        let b = u8::from_str_radix(&hex[4..6], 16).map_err(|_| "Invalid blue component")?;
        
        Ok(Color32::from_rgb(r, g, b))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_app_usage_record() {
        let now = Local::now();
        let record = AppUsageRecord::new(
            "Test App".to_string(),
            "Test Window".to_string(),
            now,
            Duration::from_secs(3600),
            "Development".to_string(),
            true,
        );

        assert_eq!(record.app_name, "Test App");
        assert_eq!(record.duration, Duration::from_secs(3600));
        assert!(record.is_productive);
    }

    #[test]
    fn test_pomodoro_record() {
        let start = Local::now();
        let end = start + chrono::Duration::minutes(25);
        let record = PomodoroRecord::new(
            start,
            end,
            PomodoroStatus::Completed,
            Some("Test note".to_string()),
            vec!["test".to_string()],
            None,
        );

        assert_eq!(record.duration().as_secs(), 25 * 60);
        assert!(matches!(record.status, PomodoroStatus::Completed));
    }

    #[test]
    fn test_daily_summary() {
        let mut summary = DailySummary::new(Local::now());
        summary.total_work_time = Duration::from_secs(3600);
        summary.productive_time = Duration::from_secs(2700);

        assert_eq!(summary.productivity_ratio(), 0.75);
    }
}