use crate::core::models::{Activity, PomodoroSession, Project};
use crate::domain::analysis::{ProductivityStats, CategoryStats, PomodoroStats};
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Default)]
pub struct AppState {
    pub current_activity: Option<Activity>,
    pub current_pomodoro: Option<PomodoroSession>,
    pub selected_project: Option<Project>,
    pub activities: Vec<Activity>,
    pub projects: Vec<Project>,
    pub pomodoro_sessions: Vec<PomodoroSession>,
    pub productivity_stats: Option<ProductivityStats>,
    pub category_stats: Vec<CategoryStats>,
    pub pomodoro_stats: Option<PomodoroStats>,
    pub daily_distribution: Vec<(u32, std::time::Duration)>,
}

pub type SharedState = Arc<RwLock<AppState>>;

impl AppState {
    pub fn new() -> SharedState {
        Arc::new(RwLock::new(AppState::default()))
    }

    pub fn set_current_activity(&mut self, activity: Option<Activity>) {
        self.current_activity = activity;
    }

    pub fn set_current_pomodoro(&mut self, session: Option<PomodoroSession>) {
        self.current_pomodoro = session;
    }

    pub fn set_selected_project(&mut self, project: Option<Project>) {
        self.selected_project = project;
    }

    pub fn update_activities(&mut self, activities: Vec<Activity>) {
        self.activities = activities;
    }

    pub fn update_projects(&mut self, projects: Vec<Project>) {
        self.projects = projects;
    }

    pub fn update_pomodoro_sessions(&mut self, sessions: Vec<PomodoroSession>) {
        self.pomodoro_sessions = sessions;
    }

    pub fn update_productivity_stats(&mut self, stats: ProductivityStats) {
        self.productivity_stats = Some(stats);
    }

    pub fn update_category_stats(&mut self, stats: Vec<CategoryStats>) {
        self.category_stats = stats;
    }

    pub fn update_pomodoro_stats(&mut self, stats: PomodoroStats) {
        self.pomodoro_stats = Some(stats);
    }

    pub fn update_daily_distribution(&mut self, distribution: Vec<(u32, std::time::Duration)>) {
        self.daily_distribution = distribution;
    }
} 