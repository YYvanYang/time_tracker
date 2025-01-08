use crate::core::{AppResult, models::*};
use crate::core::traits::{Storage, PomodoroTimer, PomodoroService};
use chrono::{DateTime, Local};
use std::sync::Arc;
use tokio::sync::RwLock;
use std::time::Duration;

pub struct PomodoroManager {
    storage: Arc<dyn Storage + Send + Sync>,
    current_session: Arc<RwLock<Option<PomodoroSession>>>,
}

impl PomodoroManager {
    pub fn new(storage: Arc<dyn Storage + Send + Sync>) -> Self {
        Self {
            storage,
            current_session: Arc::new(RwLock::new(None)),
        }
    }
}

#[async_trait::async_trait]
impl PomodoroTimer for PomodoroManager {
    async fn start_session(&self, duration: i32) -> AppResult<()> {
        let session = PomodoroSession {
            id: None,
            start_time: Local::now(),
            end_time: None,
            duration: std::time::Duration::from_secs(duration as u64 * 60),
            status: PomodoroStatus::Work,
            project_id: None,
            notes: None,
        };
        let mut current = self.current_session.write().await;
        *current = Some(session);
        Ok(())
    }

    async fn pause_session(&self) -> AppResult<()> {
        let mut current = self.current_session.write().await;
        if let Some(session) = current.as_mut() {
            session.status = PomodoroStatus::Interrupted;
        }
        Ok(())
    }

    async fn resume_session(&self) -> AppResult<()> {
        let mut current = self.current_session.write().await;
        if let Some(session) = current.as_mut() {
            session.status = PomodoroStatus::Work;
        }
        Ok(())
    }

    async fn stop_session(&self) -> AppResult<()> {
        let mut current = self.current_session.write().await;
        if let Some(mut session) = current.take() {
            session.end_time = Some(Local::now());
            session.status = PomodoroStatus::Completed;
            self.storage.save_pomodoro(&session).await?;
        }
        Ok(())
    }

    async fn get_current_session(&self) -> AppResult<Option<PomodoroSession>> {
        Ok(self.current_session.read().await.clone())
    }

    async fn is_active(&self) -> AppResult<bool> {
        Ok(self.current_session.read().await.is_some())
    }
}

#[async_trait::async_trait]
impl PomodoroService for PomodoroManager {
    async fn get_sessions(&self, start: DateTime<Local>, end: DateTime<Local>) -> AppResult<Vec<PomodoroSession>> {
        self.storage.get_pomodoro_sessions(start, end).await
    }

    async fn get_project_sessions(&self, project_id: i64, start: DateTime<Local>, end: DateTime<Local>) -> AppResult<Vec<PomodoroSession>> {
        self.storage.get_project_pomodoro_sessions(project_id, start, end).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_pomodoro_manager() {
        // TODO: 添加测试用例
    }
} 