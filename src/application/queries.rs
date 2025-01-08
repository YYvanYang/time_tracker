use crate::application::services::ServiceContainer;
use crate::core::AppResult;
use crate::core::models::*;
use crate::domain::analysis::*;
use crate::plugins::PluginRegistry;
use std::sync::Arc;

pub struct QueryHandler {
    services: Arc<ServiceContainer>,
    plugin_registry: Arc<PluginRegistry>,
}

impl QueryHandler {
    pub fn new(
        services: Arc<ServiceContainer>,
        plugin_registry: Arc<PluginRegistry>,
    ) -> Self {
        Self {
            services,
            plugin_registry,
        }
    }

    pub async fn get_projects(&self) -> AppResult<Vec<Project>> {
        self.services.storage.get_projects().await
    }

    pub async fn get_daily_activities(&self) -> AppResult<Vec<Activity>> {
        let now = chrono::Local::now();
        let start = now.date_naive().and_hms_opt(0, 0, 0).unwrap();
        let end = now.date_naive().and_hms_opt(23, 59, 59).unwrap();
        self.services.storage.get_activities(start.and_local_timezone(chrono::Local).unwrap(), end.and_local_timezone(chrono::Local).unwrap()).await
    }

    pub async fn get_productivity_stats(&self, start: chrono::DateTime<chrono::Local>, end: chrono::DateTime<chrono::Local>) -> AppResult<ProductivityStats> {
        let activities = self.services.storage.get_activities(start, end).await?;
        Ok(ProductivityStats::calculate(&activities))
    }

    pub async fn get_category_stats(&self, start: chrono::DateTime<chrono::Local>, end: chrono::DateTime<chrono::Local>) -> AppResult<Vec<CategoryStats>> {
        let activities = self.services.storage.get_activities(start, end).await?;
        Ok(CategoryStats::calculate(&activities))
    }

    pub async fn get_pomodoro_stats(&self, start: chrono::DateTime<chrono::Local>, end: chrono::DateTime<chrono::Local>) -> AppResult<PomodoroStats> {
        let pomodoros = self.services.storage.get_pomodoros(start, end).await?;
        Ok(PomodoroStats::calculate(&pomodoros))
    }

    pub async fn export_activities_csv(&self, start: chrono::DateTime<chrono::Local>, end: chrono::DateTime<chrono::Local>) -> AppResult<Vec<u8>> {
        let activities = self.services.storage.get_activities(start, end).await?;
        let mut wtr = csv::Writer::from_writer(vec![]);
        for activity in activities {
            wtr.serialize(activity)?;
        }
        Ok(wtr.into_inner()?)
    }

    pub async fn export_json(&self, start: chrono::DateTime<chrono::Local>, end: chrono::DateTime<chrono::Local>) -> AppResult<String> {
        let activities = self.services.storage.get_activities(start, end).await?;
        Ok(serde_json::to_string_pretty(&activities)?)
    }
} 