use crate::core::{AppResult, models::*};
use crate::core::traits::Storage;
use chrono::{DateTime, Local};
use std::sync::Arc;
use tokio::sync::broadcast;
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NotificationType {
    PomodoroStart,
    PomodoroEnd,
    PomodoroBreakStart,
    PomodoroBreakEnd,
    ActivityChange,
    ProductivityAlert,
    SystemAlert,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Notification {
    pub id: Option<i64>,
    pub notification_type: NotificationType,
    pub title: String,
    pub message: String,
    pub timestamp: DateTime<Local>,
    pub is_read: bool,
    pub metadata: Option<serde_json::Value>,
}

pub struct NotificationManager {
    storage: Arc<dyn Storage>,
    sender: broadcast::Sender<Notification>,
}

impl NotificationManager {
    pub fn new(storage: Arc<dyn Storage>) -> Self {
        let (sender, _) = broadcast::channel(100);
        Self { storage, sender }
    }

    pub fn subscribe(&self) -> broadcast::Receiver<Notification> {
        self.sender.subscribe()
    }

    pub async fn send_notification(&self, notification: Notification) -> AppResult<()> {
        // 保存通知到存储
        let notification = self.storage.save_notification(&notification).await?;
        
        // 广播通知
        let _ = self.sender.send(notification);
        Ok(())
    }

    pub async fn mark_as_read(&self, notification_id: i64) -> AppResult<()> {
        self.storage.mark_notification_as_read(notification_id).await
    }

    pub async fn mark_all_as_read(&self) -> AppResult<()> {
        self.storage.mark_all_notifications_as_read().await
    }

    pub async fn get_unread_notifications(&self) -> AppResult<Vec<Notification>> {
        self.storage.get_unread_notifications().await
    }

    pub async fn get_notifications(
        &self,
        start: DateTime<Local>,
        end: DateTime<Local>,
    ) -> AppResult<Vec<Notification>> {
        self.storage.get_notifications(start, end).await
    }

    pub async fn delete_notification(&self, notification_id: i64) -> AppResult<()> {
        self.storage.delete_notification(notification_id).await
    }

    pub async fn delete_old_notifications(&self, before: DateTime<Local>) -> AppResult<()> {
        self.storage.delete_old_notifications(before).await
    }

    pub async fn notify_pomodoro_start(&self, session: &PomodoroSession) -> AppResult<()> {
        let notification = Notification {
            id: None,
            notification_type: NotificationType::PomodoroStart,
            title: "番茄钟开始".into(),
            message: "新的番茄钟工作时段已开始".into(),
            timestamp: Local::now(),
            is_read: false,
            metadata: Some(serde_json::to_value(session)?),
        };
        self.send_notification(notification).await
    }

    pub async fn notify_pomodoro_end(&self, session: &PomodoroSession) -> AppResult<()> {
        let notification = Notification {
            id: None,
            notification_type: NotificationType::PomodoroEnd,
            title: "番茄钟结束".into(),
            message: "番茄钟工作时段已结束".into(),
            timestamp: Local::now(),
            is_read: false,
            metadata: Some(serde_json::to_value(session)?),
        };
        self.send_notification(notification).await
    }

    pub async fn notify_break_start(&self, duration_mins: u32) -> AppResult<()> {
        let notification = Notification {
            id: None,
            notification_type: NotificationType::PomodoroBreakStart,
            title: "休息时间开始".into(),
            message: format!("开始{}分钟的休息时间", duration_mins),
            timestamp: Local::now(),
            is_read: false,
            metadata: None,
        };
        self.send_notification(notification).await
    }

    pub async fn notify_break_end(&self) -> AppResult<()> {
        let notification = Notification {
            id: None,
            notification_type: NotificationType::PomodoroBreakEnd,
            title: "休息时间结束".into(),
            message: "休息时间已结束,准备开始新的番茄钟".into(),
            timestamp: Local::now(),
            is_read: false,
            metadata: None,
        };
        self.send_notification(notification).await
    }

    pub async fn notify_activity_change(&self, activity: &Activity) -> AppResult<()> {
        let notification = Notification {
            id: None,
            notification_type: NotificationType::ActivityChange,
            title: "活动变更".into(),
            message: format!("切换到新活动: {}", activity.window_title),
            timestamp: Local::now(),
            is_read: false,
            metadata: Some(serde_json::to_value(activity)?),
        };
        self.send_notification(notification).await
    }

    pub async fn notify_productivity_alert(
        &self,
        productive_time: std::time::Duration,
        unproductive_time: std::time::Duration,
    ) -> AppResult<()> {
        let notification = Notification {
            id: None,
            notification_type: NotificationType::ProductivityAlert,
            title: "生产力提醒".into(),
            message: format!(
                "今日已工作 {} 小时,休息 {} 小时",
                productive_time.as_secs() / 3600,
                unproductive_time.as_secs() / 3600
            ),
            timestamp: Local::now(),
            is_read: false,
            metadata: Some(serde_json::json!({
                "productive_time": productive_time.as_secs(),
                "unproductive_time": unproductive_time.as_secs(),
            })),
        };
        self.send_notification(notification).await
    }

    pub async fn notify_system_alert(&self, title: &str, message: &str) -> AppResult<()> {
        let notification = Notification {
            id: None,
            notification_type: NotificationType::SystemAlert,
            title: title.into(),
            message: message.into(),
            timestamp: Local::now(),
            is_read: false,
            metadata: None,
        };
        self.send_notification(notification).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use mockall::mock;
    use mockall::predicate::*;
    use std::time::Duration;

    mock! {
        Storage {}
        #[async_trait::async_trait]
        impl Storage for Storage {
            async fn save_notification(&self, notification: &Notification) -> AppResult<Notification>;
            async fn mark_notification_as_read(&self, id: i64) -> AppResult<()>;
            async fn mark_all_notifications_as_read(&self) -> AppResult<()>;
            async fn get_unread_notifications(&self) -> AppResult<Vec<Notification>>;
            async fn get_notifications(&self, start: DateTime<Local>, end: DateTime<Local>) -> AppResult<Vec<Notification>>;
            async fn delete_notification(&self, id: i64) -> AppResult<()>;
            async fn delete_old_notifications(&self, before: DateTime<Local>) -> AppResult<()>;
        }
    }

    #[tokio::test]
    async fn test_notification_lifecycle() -> AppResult<()> {
        let mut mock_storage = MockStorage::new();
        let now = Local::now();
        
        // 设置模拟数据
        mock_storage
            .expect_save_notification()
            .returning(|n| Ok(Notification {
                id: Some(1),
                ..n.clone()
            }));

        let manager = NotificationManager::new(Arc::new(mock_storage));
        let mut receiver = manager.subscribe();

        // 测试发送通知
        let notification = Notification {
            id: None,
            notification_type: NotificationType::SystemAlert,
            title: "Test".into(),
            message: "Test message".into(),
            timestamp: now,
            is_read: false,
            metadata: None,
        };

        manager.send_notification(notification.clone()).await?;

        // 验证接收到的通知
        if let Ok(received) = receiver.try_recv() {
            assert_eq!(received.title, "Test");
            assert_eq!(received.message, "Test message");
        }

        Ok(())
    }
} 