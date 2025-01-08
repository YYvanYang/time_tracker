use crate::core::{AppResult, models::*};
use crate::core::traits::{Storage, PomodoroTimer};
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

    async fn start_session(&self, duration: i32) -> AppResult<()> {
        let session = PomodoroSession {
            id: None,
            start_time: chrono::Local::now(),
            end_time: None,
            duration,
            status: PomodoroStatus::Work,
            project_id: None,
        };
        let mut current = self.current_session.write().await;
        *current = Some(session);
        Ok(())
    }

    async fn stop_session(&self) -> AppResult<()> {
        let mut current = self.current_session.write().await;
        if let Some(mut session) = current.take() {
            session.end_time = Some(chrono::Local::now());
            session.status = PomodoroStatus::Completed;
            self.storage.save_pomodoro(&session).await?;
        }
        Ok(())
    }

    async fn pause_session(&self) -> AppResult<()> {
        let mut current = self.current_session.write().await;
        if let Some(session) = current.as_mut() {
            session.status = PomodoroStatus::ShortBreak;
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

    async fn get_current_session(&self) -> Option<PomodoroSession> {
        self.current_session.read().await.clone()
    }

    async fn is_active(&self) -> bool {
        if let Some(session) = self.current_session.read().await.as_ref() {
            matches!(session.status, PomodoroStatus::Work)
        } else {
            false
        }
    }
}

#[async_trait::async_trait]
impl PomodoroTimer for PomodoroManager {
    async fn start_session(&self, duration: i32) -> AppResult<()> {
        self.start_session(duration).await
    }

    async fn pause_session(&self) -> AppResult<()> {
        self.pause_session().await
    }

    async fn resume_session(&self) -> AppResult<()> {
        self.resume_session().await
    }

    async fn stop_session(&self) -> AppResult<()> {
        self.stop_session().await
    }

    async fn get_current_session(&self) -> AppResult<Option<PomodoroSession>> {
        Ok(self.get_current_session().await)
    }

    async fn is_active(&self) -> AppResult<bool> {
        Ok(self.is_active().await)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use mockall::mock;
    use mockall::predicate::*;

    mock! {
        Storage {}
        #[async_trait::async_trait]
        impl Storage for Storage {
            async fn save_pomodoro_session(&self, session: &PomodoroSession) -> AppResult<i64>;
            async fn get_pomodoro_sessions(&self, start: DateTime<Local>, end: DateTime<Local>) -> AppResult<Vec<PomodoroSession>>;
            async fn get_daily_pomodoro_sessions(&self) -> AppResult<Vec<PomodoroSession>>;
            async fn get_weekly_pomodoro_sessions(&self) -> AppResult<Vec<PomodoroSession>>;
            async fn get_project_pomodoro_sessions(&self, project_id: i64, start: DateTime<Local>, end: DateTime<Local>) -> AppResult<Vec<PomodoroSession>>;
        }
    }

    #[tokio::test]
    async fn test_pomodoro_lifecycle() -> AppResult<()> {
        let mut mock_storage = MockStorage::new();
        mock_storage
            .expect_save_pomodoro_session()
            .returning(|_| Ok(1));
        
        let settings = PomodoroSettings {
            work_duration: Duration::from_secs(25 * 60),
            short_break_duration: Duration::from_secs(5 * 60),
            long_break_duration: Duration::from_secs(15 * 60),
            long_break_interval: 4,
        };

        let manager = PomodoroManager::new(Arc::new(mock_storage), settings);

        // 测试开始会话
        manager.start_session(None).await?;
        assert!(manager.is_active().await);

        // 测试获取当前会话
        let current = manager.get_current_session().await.unwrap();
        assert!(matches!(current.status, PomodoroStatus::Working));

        // 测试暂停会话
        manager.pause_session().await?;
        assert!(!manager.is_active().await);

        // 测试恢复会话
        manager.resume_session().await?;
        assert!(manager.is_active().await);

        // 测试停止会话
        tokio::time::sleep(Duration::from_secs(1)).await;
        manager.stop_session().await?;
        assert!(!manager.is_active().await);

        Ok(())
    }
} 