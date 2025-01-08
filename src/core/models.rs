use chrono::{DateTime, Local};
use serde::{Deserialize, Serialize};
use std::time::Duration;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Activity {
    pub id: Option<i64>,
    pub name: String,
    pub start_time: DateTime<Local>,
    pub end_time: Option<DateTime<Local>>,
    pub project_id: Option<i64>,
    pub description: Option<String>,
    pub duration: Duration,
    pub category: String,
    pub is_productive: bool,
    pub app_name: String,
    pub window_title: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
pub struct Project {
    pub id: Option<i64>,
    pub name: String,
    pub description: Option<String>,
    pub created_at: DateTime<Local>,
    pub updated_at: DateTime<Local>,
}

impl std::fmt::Display for Project {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name)
    }
}

impl Project {
    pub fn new(name: String, description: Option<String>) -> Self {
        let now = Local::now();
        Self {
            id: None,
            name,
            description,
            created_at: now,
            updated_at: now,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PomodoroSession {
    pub id: Option<i64>,
    pub start_time: DateTime<Local>,
    pub end_time: Option<DateTime<Local>>,
    pub duration: Duration,
    pub status: PomodoroStatus,
    pub project_id: Option<i64>,
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PomodoroStatus {
    Work,
    ShortBreak,
    LongBreak,
    Completed,
    Interrupted,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tag {
    pub id: Option<i64>,
    pub name: String,
    pub color: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppState {
    pub current_activity: Option<Activity>,
    pub current_pomodoro: Option<PomodoroSession>,
    pub is_tracking: bool,
    pub last_update: DateTime<Local>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DailySummary {
    pub date: DateTime<Local>,
    pub total_time: Duration,
    pub productive_time: Duration,
    pub activities: Vec<Activity>,
    pub pomodoros: Vec<PomodoroSession>,
    pub projects: Vec<ProjectSummary>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WeeklySummary {
    pub start_date: DateTime<Local>,
    pub end_date: DateTime<Local>,
    pub total_time: Duration,
    pub productive_time: Duration,
    pub daily_summaries: Vec<DailySummary>,
    pub projects: Vec<ProjectSummary>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonthlySummary {
    pub month: DateTime<Local>,
    pub total_time: Duration,
    pub productive_time: Duration,
    pub weekly_summaries: Vec<WeeklySummary>,
    pub projects: Vec<ProjectSummary>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectSummary {
    pub project: Project,
    pub total_time: Duration,
    pub activities_count: usize,
    pub pomodoros_count: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ExportFormat {
    CSV,
    JSON,
    Excel,
}

#[derive(Debug, Clone)]
pub struct ProductivityStats {
    pub total_time: i64,
    pub productive_time: i64,
    pub productivity_score: f64,
}

#[derive(Debug, Clone)]
pub struct CategoryStats {
    pub category: String,
    pub total_time: i64,
    pub percentage: f64,
}

#[derive(Debug, Clone)]
pub struct PomodoroStats {
    pub total_sessions: i32,
    pub completed_sessions: i32,
    pub total_time: i64,
    pub average_duration: i64,
    pub completion_rate: f64,
}

impl ProductivityStats {
    pub fn calculate(activities: &[Activity]) -> Self {
        let total_time = activities.iter().map(|a| a.duration).sum();
        let productive_time = activities.iter()
            .filter(|a| a.is_productive)
            .map(|a| a.duration)
            .sum();
        let productivity_score = if total_time > 0 {
            productive_time as f64 / total_time as f64
        } else {
            0.0
        };
        
        Self {
            total_time,
            productive_time,
            productivity_score,
        }
    }
}

impl CategoryStats {
    pub fn calculate(activities: &[Activity]) -> Vec<Self> {
        let mut categories = std::collections::HashMap::new();
        let total_time: i64 = activities.iter().map(|a| a.duration).sum();
        
        for activity in activities {
            let entry = categories.entry(activity.category.clone())
                .or_insert(0i64);
            *entry += activity.duration;
        }
        
        categories.into_iter()
            .map(|(category, time)| {
                let percentage = if total_time > 0 {
                    time as f64 / total_time as f64
                } else {
                    0.0
                };
                
                Self {
                    category,
                    total_time: time,
                    percentage,
                }
            })
            .collect()
    }
}

impl PomodoroStats {
    pub fn calculate(sessions: &[PomodoroSession]) -> Self {
        let total_sessions = sessions.len() as i32;
        let completed_sessions = sessions.iter()
            .filter(|s| matches!(s.status, PomodoroStatus::Completed))
            .count() as i32;
        let total_time = sessions.iter()
            .map(|s| s.duration)
            .sum();
        let average_duration = if total_sessions > 0 {
            total_time / total_sessions as i64
        } else {
            0
        };
        let completion_rate = if total_sessions > 0 {
            completed_sessions as f64 / total_sessions as f64
        } else {
            0.0
        };
        
        Self {
            total_sessions,
            completed_sessions,
            total_time,
            average_duration,
            completion_rate,
        }
    }
} 