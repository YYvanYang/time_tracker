use crate::core::{AppResult, models::*};
use crate::core::traits::{Storage, PomodoroTimer};
use chrono::{DateTime, Local};
use std::sync::Arc;
use tokio::sync::RwLock;
use std::time::Duration;

pub struct PomodoroManager {
    storage: Arc<dyn Storage>,
    current_session: RwLock<Option<PomodoroSession>>,
    start_time: RwLock<Option<DateTime<Local>>>,
    settings: RwLock<PomodoroSettings>,
}

impl PomodoroManager {
    pub fn new(storage: Arc<dyn Storage>, settings: PomodoroSettings) -> Self {
        Self {
            storage,
            current_session: RwLock::new(None),
            start_time: RwLock::new(None),
            settings: RwLock::new(settings),
        }
    }

    pub async fn start_session(&self, project_id: Option<i64>) -> AppResult<()> {
        let now = Local::now();
        
        // 如果有正在进行的会话,先保存它
        if let Some(mut session) = self.current_session.write().await.take() {
            if let Some(start) = *self.start_time.read().await {
                session.duration = now.signed_duration_since(start).to_std()?;
                self.storage.save_pomodoro_session(&session).await?;
            }
        }

        // 创建新会话
        let session = PomodoroSession {
            id: None,
            start_time: now,
            duration: Duration::from_secs(0),
            status: PomodoroStatus::Working,
            project_id,
            notes: None,
        };

        *self.current_session.write().await = Some(session);
        *self.start_time.write().await = Some(now);

        Ok(())
    }

    pub async fn stop_session(&self) -> AppResult<()> {
        let now = Local::now();
        
        if let Some(mut session) = self.current_session.write().await.take() {
            if let Some(start) = self.start_time.write().await.take() {
                session.duration = now.signed_duration_since(start).to_std()?;
                session.status = PomodoroStatus::Completed;
                self.storage.save_pomodoro_session(&session).await?;
            }
        }

        Ok(())
    }

    pub async fn pause_session(&self) -> AppResult<()> {
        if let Some(mut session) = self.current_session.write().await.as_mut() {
            session.status = PomodoroStatus::Paused;
            if let Some(start) = *self.start_time.read().await {
                session.duration = Local::now().signed_duration_since(start).to_std()?;
            }
        }
        Ok(())
    }

    pub async fn resume_session(&self) -> AppResult<()> {
        if let Some(mut session) = self.current_session.write().await.as_mut() {
            session.status = PomodoroStatus::Working;
            *self.start_time.write().await = Some(Local::now());
        }
        Ok(())
    }

    pub async fn get_current_session(&self) -> Option<PomodoroSession> {
        self.current_session.read().await.clone()
    }

    pub async fn get_elapsed_time(&self) -> Duration {
        if let Some(start) = *self.start_time.read().await {
            Local::now()
                .signed_duration_since(start)
                .to_std()
                .unwrap_or(Duration::from_secs(0))
        } else {
            Duration::from_secs(0)
        }
    }

    pub async fn update_settings(&self, settings: PomodoroSettings) -> AppResult<()> {
        *self.settings.write().await = settings;
        Ok(())
    }

    pub async fn get_settings(&self) -> PomodoroSettings {
        self.settings.read().await.clone()
    }

    pub async fn add_session_note(&self, note: String) -> AppResult<()> {
        if let Some(mut session) = self.current_session.write().await.as_mut() {
            session.notes = Some(note);
        }
        Ok(())
    }

    pub async fn get_sessions_by_date_range(
        &self,
        start: DateTime<Local>,
        end: DateTime<Local>,
    ) -> AppResult<Vec<PomodoroSession>> {
        self.storage.get_pomodoro_sessions(start, end).await
    }

    pub async fn get_daily_sessions(&self) -> AppResult<Vec<PomodoroSession>> {
        self.storage.get_daily_pomodoro_sessions().await
    }

    pub async fn get_weekly_sessions(&self) -> AppResult<Vec<PomodoroSession>> {
        self.storage.get_weekly_pomodoro_sessions().await
    }

    pub async fn get_project_sessions(
        &self,
        project_id: i64,
        start: DateTime<Local>,
        end: DateTime<Local>,
    ) -> AppResult<Vec<PomodoroSession>> {
        self.storage.get_project_pomodoro_sessions(project_id, start, end).await
    }
}

#[async_trait::async_trait]
impl PomodoroTimer for PomodoroManager {
    async fn start(&self) -> AppResult<()> {
        self.start_session(None).await
    }

    async fn stop(&self) -> AppResult<()> {
        self.stop_session().await
    }

    async fn pause(&self) -> AppResult<()> {
        self.pause_session().await
    }

    async fn resume(&self) -> AppResult<()> {
        self.resume_session().await
    }

    async fn is_active(&self) -> bool {
        if let Some(session) = self.current_session.read().await.as_ref() {
            matches!(session.status, PomodoroStatus::Working)
        } else {
            false
        }
    }

    async fn get_remaining_time(&self) -> Duration {
        let settings = self.settings.read().await;
        let elapsed = self.get_elapsed_time().await;
        if elapsed >= settings.work_duration {
            Duration::from_secs(0)
        } else {
            settings.work_duration - elapsed
        }
    }

    async fn get_current_session(&self) -> AppResult<Option<PomodoroSession>> {
        Ok(self.get_current_session().await)
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