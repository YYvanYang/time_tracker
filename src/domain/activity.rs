use crate::core::{AppResult, models::*};
use crate::core::traits::{Storage, TimeTracker};
use chrono::{DateTime, Local, Duration as ChronoDuration};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;

pub struct ActivityManager {
    storage: Arc<dyn Storage>,
    current_activity: RwLock<Option<Activity>>,
    start_time: RwLock<Option<DateTime<Local>>>,
}

impl ActivityManager {
    pub fn new(storage: Arc<dyn Storage>) -> Self {
        Self {
            storage,
            current_activity: RwLock::new(None),
            start_time: RwLock::new(None),
        }
    }

    pub async fn start_activity(&self, app_name: String, window_title: String) -> AppResult<()> {
        let now = Local::now();
        
        // 如果有正在进行的活动,先保存它
        if let Some(mut activity) = self.current_activity.write().await.take() {
            if let Some(start) = *self.start_time.read().await {
                activity.duration = now.signed_duration_since(start).to_std()?;
                self.storage.save_activity(&activity).await?;
            }
        }

        // 创建新活动
        let activity = Activity {
            id: None,
            app_name,
            window_title,
            start_time: now,
            duration: Duration::from_secs(0),
            category: None, // 可以通过规则引擎自动分类
            is_productive: false, // 可以通过规则引擎判断
            project_id: None,
        };

        *self.current_activity.write().await = Some(activity);
        *self.start_time.write().await = Some(now);

        Ok(())
    }

    pub async fn stop_activity(&self) -> AppResult<()> {
        let now = Local::now();
        
        if let Some(mut activity) = self.current_activity.write().await.take() {
            if let Some(start) = self.start_time.write().await.take() {
                activity.duration = now.signed_duration_since(start).to_std()?;
                self.storage.save_activity(&activity).await?;
            }
        }

        Ok(())
    }

    pub async fn get_current_activity(&self) -> Option<Activity> {
        self.current_activity.read().await.clone()
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

    pub async fn get_activities_by_date_range(
        &self,
        start: DateTime<Local>,
        end: DateTime<Local>,
    ) -> AppResult<Vec<Activity>> {
        self.storage.get_activities(start, end).await
    }

    pub async fn get_daily_activities(&self) -> AppResult<Vec<Activity>> {
        self.storage.get_daily_activities().await
    }

    pub async fn get_weekly_activities(&self) -> AppResult<Vec<Activity>> {
        self.storage.get_weekly_activities().await
    }

    pub async fn update_activity_category(
        &self,
        activity_id: i64,
        category: Option<String>,
    ) -> AppResult<()> {
        if let Some(mut activity) = self.current_activity.write().await.as_mut() {
            if activity.id == Some(activity_id) {
                activity.category = category;
            }
        }
        Ok(())
    }

    pub async fn update_activity_productivity(
        &self,
        activity_id: i64,
        is_productive: bool,
    ) -> AppResult<()> {
        if let Some(mut activity) = self.current_activity.write().await.as_mut() {
            if activity.id == Some(activity_id) {
                activity.is_productive = is_productive;
            }
        }
        Ok(())
    }

    pub async fn assign_to_project(
        &self,
        activity_id: i64,
        project_id: Option<i64>,
    ) -> AppResult<()> {
        if let Some(mut activity) = self.current_activity.write().await.as_mut() {
            if activity.id == Some(activity_id) {
                activity.project_id = project_id;
            }
        }
        Ok(())
    }
}

#[async_trait::async_trait]
impl TimeTracker for ActivityManager {
    async fn start_tracking(&self) -> AppResult<()> {
        // 实现开始追踪的逻辑
        Ok(())
    }

    async fn stop_tracking(&self) -> AppResult<()> {
        self.stop_activity().await
    }

    async fn is_tracking(&self) -> bool {
        self.current_activity.read().await.is_some()
    }

    async fn get_current_activity(&self) -> AppResult<Option<Activity>> {
        Ok(self.get_current_activity().await)
    }

    async fn get_elapsed_time(&self) -> Duration {
        self.get_elapsed_time().await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use mockall::mock;
    use mockall::predicate::*;
    use std::sync::Arc;

    mock! {
        Storage {}
        #[async_trait::async_trait]
        impl Storage for Storage {
            async fn save_activity(&self, activity: &Activity) -> AppResult<i64>;
            async fn get_activities(&self, start: DateTime<Local>, end: DateTime<Local>) -> AppResult<Vec<Activity>>;
            async fn get_daily_activities(&self) -> AppResult<Vec<Activity>>;
            async fn get_weekly_activities(&self) -> AppResult<Vec<Activity>>;
            async fn delete_activity(&self, id: i64) -> AppResult<()>;
        }
    }

    #[tokio::test]
    async fn test_activity_lifecycle() -> AppResult<()> {
        let mut mock_storage = MockStorage::new();
        mock_storage
            .expect_save_activity()
            .returning(|_| Ok(1));
        
        let manager = ActivityManager::new(Arc::new(mock_storage));

        // 测试开始活动
        manager.start_activity("test_app".into(), "test_window".into()).await?;
        assert!(manager.is_tracking().await);

        // 测试获取当前活动
        let current = manager.get_current_activity().await.unwrap();
        assert_eq!(current.app_name, "test_app");
        assert_eq!(current.window_title, "test_window");

        // 测试停止活动
        tokio::time::sleep(Duration::from_secs(1)).await;
        manager.stop_activity().await?;
        assert!(!manager.is_tracking().await);

        Ok(())
    }
} 