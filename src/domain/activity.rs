use std::sync::Arc;
use tokio::sync::RwLock;
use chrono::{DateTime, Local};
use crate::core::{AppResult, models::*};
use crate::core::traits::*;

pub struct ActivityManager {
    storage: Arc<dyn Storage + Send + Sync>,
    current_activity: Arc<RwLock<Option<Activity>>>,
}

impl ActivityManager {
    pub fn new(storage: Arc<dyn Storage + Send + Sync>) -> Self {
        Self {
            storage,
            current_activity: Arc::new(RwLock::new(None)),
        }
    }

    async fn start_activity(&self, activity: Activity) -> AppResult<()> {
        let mut current = self.current_activity.write().await;
        *current = Some(activity);
        Ok(())
    }

    async fn stop_activity(&self) -> AppResult<()> {
        let mut current = self.current_activity.write().await;
        *current = None;
        Ok(())
    }

    async fn get_current_activity(&self) -> Option<Activity> {
        self.current_activity.read().await.clone()
    }

    async fn get_elapsed_time(&self) -> std::time::Duration {
        if let Some(activity) = self.current_activity.read().await.as_ref() {
            chrono::Local::now()
                .signed_duration_since(activity.start_time)
                .to_std()
                .unwrap_or_default()
        } else {
            std::time::Duration::from_secs(0)
        }
    }
}

#[async_trait::async_trait]
impl TimeTracker for ActivityManager {
    async fn start_tracking(&self, activity: Activity) -> AppResult<()> {
        self.start_activity(activity).await
    }

    async fn stop_tracking(&self) -> AppResult<()> {
        self.stop_activity().await
    }

    async fn get_current_activity(&self) -> AppResult<Option<Activity>> {
        Ok(self.get_current_activity().await)
    }

    async fn is_tracking(&self) -> AppResult<bool> {
        Ok(self.current_activity.read().await.is_some())
    }
}

#[async_trait::async_trait]
impl ActivityService for ActivityManager {
    async fn get_activities(&self, start: DateTime<Local>, end: DateTime<Local>) -> AppResult<Vec<Activity>> {
        self.storage.get_activities(start, end).await
    }

    async fn get_project_activities(&self, project_id: i64, start: DateTime<Local>, end: DateTime<Local>) -> AppResult<Vec<Activity>> {
        self.storage.get_project_activities(project_id, start, end).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_activity_manager() {
        // TODO: 添加测试用例
    }
} 