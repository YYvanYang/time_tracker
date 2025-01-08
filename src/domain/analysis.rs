use crate::core::models::{Activity, PomodoroSession};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ProductivityStats {
    pub total_time: std::time::Duration,
    pub productive_time: std::time::Duration,
    pub unproductive_time: std::time::Duration,
    pub productivity_score: f64,
}

impl ProductivityStats {
    pub fn calculate(activities: &[Activity]) -> Self {
        let mut stats = Self::default();
        
        for activity in activities {
            stats.total_time += activity.duration;
            if activity.is_productive {
                stats.productive_time += activity.duration;
            } else {
                stats.unproductive_time += activity.duration;
            }
        }
        
        if !stats.total_time.is_zero() {
            stats.productivity_score = stats.productive_time.as_secs_f64() / stats.total_time.as_secs_f64() * 100.0;
        }
        
        stats
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CategoryStats {
    pub category: String,
    pub total_time: std::time::Duration,
    pub activity_count: usize,
}

impl CategoryStats {
    pub fn calculate(activities: &[Activity]) -> Vec<Self> {
        let mut category_map: HashMap<String, CategoryStats> = HashMap::new();
        
        for activity in activities {
            let stats = category_map.entry(activity.category.clone()).or_default();
            stats.category = activity.category.clone();
            stats.total_time += activity.duration;
            stats.activity_count += 1;
        }
        
        let mut stats: Vec<_> = category_map.into_values().collect();
        stats.sort_by(|a, b| b.total_time.cmp(&a.total_time));
        stats
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PomodoroStats {
    pub total_sessions: usize,
    pub completed_sessions: usize,
    pub interrupted_sessions: usize,
    pub total_work_time: std::time::Duration,
    pub total_break_time: std::time::Duration,
    pub completion_rate: f64,
}

impl PomodoroStats {
    pub fn calculate(sessions: &[PomodoroSession]) -> Self {
        let mut stats = Self::default();
        
        stats.total_sessions = sessions.len();
        
        for session in sessions {
            match session.status {
                crate::core::models::PomodoroStatus::Completed => {
                    stats.completed_sessions += 1;
                    stats.total_work_time += session.duration;
                }
                crate::core::models::PomodoroStatus::Interrupted => {
                    stats.interrupted_sessions += 1;
                }
                crate::core::models::PomodoroStatus::ShortBreak | crate::core::models::PomodoroStatus::LongBreak => {
                    stats.total_break_time += session.duration;
                }
                _ => {}
            }
        }
        
        if stats.total_sessions > 0 {
            stats.completion_rate = stats.completed_sessions as f64 / stats.total_sessions as f64 * 100.0;
        }
        
        stats
    }
} 