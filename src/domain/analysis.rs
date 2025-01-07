use crate::core::{AppResult, models::*};
use crate::core::traits::Storage;
use chrono::{DateTime, Local, Duration as ChronoDuration, Datelike, Timelike};
use std::sync::Arc;
use std::collections::HashMap;
use std::time::Duration;

pub struct AnalysisManager {
    storage: Arc<dyn Storage>,
}

impl AnalysisManager {
    pub fn new(storage: Arc<dyn Storage>) -> Self {
        Self { storage }
    }

    pub async fn get_daily_summary(&self, date: DateTime<Local>) -> AppResult<DailySummary> {
        let start = date.date_naive().and_hms_opt(0, 0, 0).unwrap()
            .and_local_timezone(Local).unwrap();
        let end = date.date_naive().and_hms_opt(23, 59, 59).unwrap()
            .and_local_timezone(Local).unwrap();

        let activities = self.storage.get_activities(start, end).await?;
        let pomodoros = self.storage.get_pomodoro_sessions(start, end).await?;

        let total_work_time = activities.iter()
            .filter(|a| a.is_productive)
            .fold(Duration::from_secs(0), |acc, a| acc + a.duration);

        let total_break_time = activities.iter()
            .filter(|a| !a.is_productive)
            .fold(Duration::from_secs(0), |acc, a| acc + a.duration);

        let completed_pomodoros = pomodoros.iter()
            .filter(|p| matches!(p.status, PomodoroStatus::Completed))
            .count();

        let productivity_score = if total_work_time.as_secs() + total_break_time.as_secs() > 0 {
            total_work_time.as_secs_f64() / (total_work_time.as_secs_f64() + total_break_time.as_secs_f64())
        } else {
            0.0
        };

        Ok(DailySummary {
            date,
            total_work_time,
            total_break_time,
            completed_pomodoros,
            productivity_score,
        })
    }

    pub async fn get_weekly_summary(&self) -> AppResult<WeeklySummary> {
        let now = Local::now();
        let start = now.date_naive().and_hms_opt(0, 0, 0).unwrap()
            .and_local_timezone(Local).unwrap() - ChronoDuration::days(7);
        let end = now;

        let mut daily_summaries = Vec::new();
        let mut current = start;

        while current <= end {
            daily_summaries.push(self.get_daily_summary(current).await?);
            current = current + ChronoDuration::days(1);
        }

        let total_work_time = daily_summaries.iter()
            .fold(Duration::from_secs(0), |acc, s| acc + s.total_work_time);

        let total_break_time = daily_summaries.iter()
            .fold(Duration::from_secs(0), |acc, s| acc + s.total_break_time);

        let completed_pomodoros = daily_summaries.iter()
            .fold(0, |acc, s| acc + s.completed_pomodoros);

        let avg_productivity_score = daily_summaries.iter()
            .fold(0.0, |acc, s| acc + s.productivity_score) / daily_summaries.len() as f64;

        Ok(WeeklySummary {
            start_date: start,
            end_date: end,
            daily_summaries,
            total_work_time,
            total_break_time,
            completed_pomodoros,
            avg_productivity_score,
        })
    }

    pub async fn get_productivity_by_hour(&self, date: DateTime<Local>) -> AppResult<HashMap<u32, f64>> {
        let start = date.date_naive().and_hms_opt(0, 0, 0).unwrap()
            .and_local_timezone(Local).unwrap();
        let end = date.date_naive().and_hms_opt(23, 59, 59).unwrap()
            .and_local_timezone(Local).unwrap();

        let activities = self.storage.get_activities(start, end).await?;
        let mut productivity_by_hour = HashMap::new();

        for activity in activities {
            let hour = activity.start_time.hour();
            let entry = productivity_by_hour.entry(hour).or_insert(0.0);
            if activity.is_productive {
                *entry += activity.duration.as_secs_f64();
            }
        }

        // 将秒数转换为小时的百分比
        for value in productivity_by_hour.values_mut() {
            *value = *value / 3600.0;
        }

        Ok(productivity_by_hour)
    }

    pub async fn get_activity_distribution(&self, date: DateTime<Local>) -> AppResult<HashMap<String, Duration>> {
        let start = date.date_naive().and_hms_opt(0, 0, 0).unwrap()
            .and_local_timezone(Local).unwrap();
        let end = date.date_naive().and_hms_opt(23, 59, 59).unwrap()
            .and_local_timezone(Local).unwrap();

        let activities = self.storage.get_activities(start, end).await?;
        let mut distribution = HashMap::new();

        for activity in activities {
            let category = activity.category.unwrap_or_else(|| "未分类".to_string());
            let entry = distribution.entry(category).or_insert(Duration::from_secs(0));
            *entry += activity.duration;
        }

        Ok(distribution)
    }

    pub async fn get_project_stats(&self, project_id: i64) -> AppResult<ProjectStats> {
        let now = Local::now();
        let start = now.date_naive().and_hms_opt(0, 0, 0).unwrap()
            .and_local_timezone(Local).unwrap() - ChronoDuration::days(30);
        let end = now;

        let activities = self.storage.get_project_activities(project_id, start, end).await?;
        let pomodoros = self.storage.get_project_pomodoro_sessions(project_id, start, end).await?;

        let total_time = activities.iter()
            .fold(Duration::from_secs(0), |acc, a| acc + a.duration);

        let productive_time = activities.iter()
            .filter(|a| a.is_productive)
            .fold(Duration::from_secs(0), |acc, a| acc + a.duration);

        let completed_pomodoros = pomodoros.iter()
            .filter(|p| matches!(p.status, PomodoroStatus::Completed))
            .count();

        Ok(ProjectStats {
            project_id,
            total_time,
            productive_time,
            completed_pomodoros,
            period_start: start,
            period_end: end,
        })
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
            async fn get_activities(&self, start: DateTime<Local>, end: DateTime<Local>) -> AppResult<Vec<Activity>>;
            async fn get_pomodoro_sessions(&self, start: DateTime<Local>, end: DateTime<Local>) -> AppResult<Vec<PomodoroSession>>;
            async fn get_project_activities(&self, project_id: i64, start: DateTime<Local>, end: DateTime<Local>) -> AppResult<Vec<Activity>>;
            async fn get_project_pomodoro_sessions(&self, project_id: i64, start: DateTime<Local>, end: DateTime<Local>) -> AppResult<Vec<PomodoroSession>>;
        }
    }

    #[tokio::test]
    async fn test_daily_summary() -> AppResult<()> {
        let mut mock_storage = MockStorage::new();
        let now = Local::now();
        
        // 设置模拟数据
        mock_storage
            .expect_get_activities()
            .returning(|_, _| Ok(vec![
                Activity {
                    id: Some(1),
                    app_name: "test_app".into(),
                    window_title: "test_window".into(),
                    start_time: now,
                    duration: Duration::from_secs(3600),
                    category: Some("work".into()),
                    is_productive: true,
                    project_id: None,
                }
            ]));
        
        mock_storage
            .expect_get_pomodoro_sessions()
            .returning(|_, _| Ok(vec![
                PomodoroSession {
                    id: Some(1),
                    start_time: now,
                    duration: Duration::from_secs(1500),
                    status: PomodoroStatus::Completed,
                    project_id: None,
                    notes: None,
                }
            ]));

        let manager = AnalysisManager::new(Arc::new(mock_storage));
        let summary = manager.get_daily_summary(now).await?;

        assert_eq!(summary.total_work_time, Duration::from_secs(3600));
        assert_eq!(summary.completed_pomodoros, 1);
        assert!(summary.productivity_score > 0.0);

        Ok(())
    }
} 