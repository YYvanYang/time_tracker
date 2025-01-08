use serde::{Deserialize, Serialize};
use crate::core::AppResult;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub database_url: String,
    pub auto_start: bool,
    pub minimize_to_tray: bool,
    pub idle_threshold: u64,
    pub pomodoro_duration: u64,
    pub short_break_duration: u64,
    pub long_break_duration: u64,
    pub long_break_interval: u32,
}

impl Config {
    pub fn validate(&self) -> AppResult<()> {
        if self.database_url.is_empty() {
            return Err(crate::core::AppError::Validation("数据库 URL 不能为空".into()));
        }
        if self.pomodoro_duration == 0 {
            return Err(crate::core::AppError::Validation("番茄钟时长必须大于0".into()));
        }
        if self.short_break_duration == 0 {
            return Err(crate::core::AppError::Validation("短休息时长必须大于0".into()));
        }
        if self.long_break_duration == 0 {
            return Err(crate::core::AppError::Validation("长休息时长必须大于0".into()));
        }
        if self.long_break_interval == 0 {
            return Err(crate::core::AppError::Validation("长休息间隔必须大于0".into()));
        }
        Ok(())
    }

    pub fn default() -> Self {
        Self {
            database_url: "sqlite:time_tracker.db".into(),
            auto_start: true,
            minimize_to_tray: true,
            idle_threshold: 300,
            pomodoro_duration: 25 * 60,
            short_break_duration: 5 * 60,
            long_break_duration: 15 * 60,
            long_break_interval: 4,
        }
    }
} 