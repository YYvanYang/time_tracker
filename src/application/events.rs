use crate::core::models::{Activity, PomodoroSession, Project};
use chrono::{DateTime, Local};
use std::sync::Arc;
use tokio::sync::broadcast;

#[derive(Debug, Clone)]
pub enum AppEvent {
    // 活动事件
    ActivityStarted(Arc<Activity>),
    ActivityStopped(Arc<Activity>),
    ActivityUpdated(Arc<Activity>),
    
    // 项目事件
    ProjectCreated(Arc<Project>),
    ProjectUpdated(Arc<Project>),
    ProjectDeleted(i64),
    
    // 番茄钟事件
    PomodoroStarted(Arc<PomodoroSession>),
    PomodoroPaused(Arc<PomodoroSession>),
    PomodoroResumed(Arc<PomodoroSession>),
    PomodoroCompleted(Arc<PomodoroSession>),
    PomodoroInterrupted(Arc<PomodoroSession>),
    PomodoroTick { session_id: i64, elapsed: std::time::Duration },
    
    // 配置事件
    ConfigUpdated,
    
    // 系统事件
    ApplicationStarted,
    ApplicationStopping,
    BackupStarted,
    BackupCompleted,
    DatabaseVacuumStarted,
    DatabaseVacuumCompleted,
    
    // 窗口事件
    WindowShown,
    WindowHidden,
}

#[derive(Clone)]
pub struct EventBus {
    sender: broadcast::Sender<AppEvent>,
}

impl EventBus {
    pub fn new(capacity: usize) -> Self {
        let (sender, _) = broadcast::channel(capacity);
        Self { sender }
    }

    pub fn subscribe(&self) -> broadcast::Receiver<AppEvent> {
        self.sender.subscribe()
    }

    pub fn publish(&self, event: AppEvent) {
        let _ = self.sender.send(event);
    }
}

impl Default for EventBus {
    fn default() -> Self {
        Self::new(32)
    }
} 