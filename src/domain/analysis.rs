use std::sync::Arc;
use chrono::{DateTime, Local, Datelike};
use crate::core::{AppResult, models::*, traits::*};

pub struct AnalysisManager {
    storage: Arc<dyn Storage + Send + Sync>,
}

impl AnalysisManager {
    pub fn new(storage: Arc<dyn Storage + Send + Sync>) -> Self {
        Self { storage }
    }

    async fn calculate_project_summaries(&self, activities: &[Activity], pomodoros: &[PomodoroSession]) -> AppResult<Vec<ProjectSummary>> {
        let mut project_summaries = Vec::new();
        let projects = self.storage.list_projects().await?;

        for project in projects {
            let project_activities: Vec<_> = activities.iter()
                .filter(|a| a.project_id == project.id)
                .collect();
            
            let project_pomodoros: Vec<_> = pomodoros.iter()
                .filter(|p| p.project_id == project.id)
                .collect();

            let total_time = project_activities.iter()
                .map(|a| a.duration)
                .sum();

            project_summaries.push(ProjectSummary {
                project,
                total_time,
                activities_count: project_activities.len(),
                pomodoros_count: project_pomodoros.len(),
            });
        }

        Ok(project_summaries)
    }
}

#[async_trait::async_trait]
impl AnalysisService for AnalysisManager {
    async fn get_daily_summary(&self, date: DateTime<Local>) -> AppResult<DailySummary> {
        let start = date.date_naive().and_hms_opt(0, 0, 0).unwrap();
        let end = date.date_naive().and_hms_opt(23, 59, 59).unwrap();
        let start = DateTime::<Local>::from_naive_utc_and_offset(start, *Local::now().offset());
        let end = DateTime::<Local>::from_naive_utc_and_offset(end, *Local::now().offset());

        let activities = self.storage.get_activities(start, end).await?;
        let pomodoros = self.storage.get_pomodoro_sessions(start, end).await?;

        let total_time: std::time::Duration = activities.iter()
            .map(|a| a.duration)
            .sum();

        let productive_time: std::time::Duration = activities.iter()
            .filter(|a| a.is_productive)
            .map(|a| a.duration)
            .sum();

        let projects = self.calculate_project_summaries(&activities, &pomodoros).await?;

        Ok(DailySummary {
            date,
            total_time,
            productive_time,
            activities,
            pomodoros,
            projects,
        })
    }

    async fn get_weekly_summary(&self, start: DateTime<Local>) -> AppResult<WeeklySummary> {
        let end = start + chrono::Duration::days(7);
        let mut daily_summaries = Vec::new();
        let mut current = start;

        while current < end {
            daily_summaries.push(self.get_daily_summary(current).await?);
            current = current + chrono::Duration::days(1);
        }

        let total_time: std::time::Duration = daily_summaries.iter()
            .map(|s| s.total_time)
            .sum();

        let productive_time: std::time::Duration = daily_summaries.iter()
            .map(|s| s.productive_time)
            .sum();

        let all_activities: Vec<_> = daily_summaries.iter()
            .flat_map(|s| s.activities.clone())
            .collect();

        let all_pomodoros: Vec<_> = daily_summaries.iter()
            .flat_map(|s| s.pomodoros.clone())
            .collect();

        let projects = self.calculate_project_summaries(&all_activities, &all_pomodoros).await?;

        Ok(WeeklySummary {
            start_date: start,
            end_date: end,
            total_time,
            productive_time,
            daily_summaries,
            projects,
        })
    }

    async fn get_monthly_summary(&self, start: DateTime<Local>) -> AppResult<MonthlySummary> {
        let mut weekly_summaries = Vec::new();
        let mut current = start;
        let days_in_month = if start.month() == 12 {
            31
        } else {
            Local.with_ymd_and_hms(start.year(), start.month() + 1, 1, 0, 0, 0)
                .unwrap()
                .signed_duration_since(Local.with_ymd_and_hms(start.year(), start.month(), 1, 0, 0, 0).unwrap())
                .num_days() as i64
        };

        for _ in 0..(days_in_month / 7 + 1) {
            if current < start + chrono::Duration::days(days_in_month) {
                weekly_summaries.push(self.get_weekly_summary(current).await?);
                current = current + chrono::Duration::days(7);
            }
        }

        let total_time: std::time::Duration = weekly_summaries.iter()
            .map(|s| s.total_time)
            .sum();

        let productive_time: std::time::Duration = weekly_summaries.iter()
            .map(|s| s.productive_time)
            .sum();

        let all_activities: Vec<_> = weekly_summaries.iter()
            .flat_map(|s| s.daily_summaries.iter())
            .flat_map(|s| s.activities.clone())
            .collect();

        let all_pomodoros: Vec<_> = weekly_summaries.iter()
            .flat_map(|s| s.daily_summaries.iter())
            .flat_map(|s| s.pomodoros.clone())
            .collect();

        let projects = self.calculate_project_summaries(&all_activities, &all_pomodoros).await?;

        Ok(MonthlySummary {
            month: start,
            total_time,
            productive_time,
            weekly_summaries,
            projects,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_analysis_manager() {
        // TODO: 添加测试用例
    }
} 