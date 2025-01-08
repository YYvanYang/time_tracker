use crate::core::{AppResult, models::*};
use crate::core::traits::Storage;
use chrono::{DateTime, Local};
use std::sync::Arc;
use serde_json;
use csv;
use std::time::Duration;

pub struct ExportManager {
    storage: Arc<dyn Storage + Send + Sync>,
}

impl ExportManager {
    pub fn new(storage: Arc<dyn Storage + Send + Sync>) -> Self {
        Self { storage }
    }

    fn format_duration(duration: std::time::Duration) -> String {
        let total_seconds = duration.as_secs();
        let hours = total_seconds / 3600;
        let minutes = (total_seconds % 3600) / 60;
        let seconds = total_seconds % 60;
        format!("{:02}:{:02}:{:02}", hours, minutes, seconds)
    }

    async fn export_activities_to_csv(&self, activities: &[Activity]) -> AppResult<Vec<u8>> {
        let mut wtr = csv::Writer::from_writer(Vec::new());
        
        wtr.write_record(&[
            "ID",
            "Name",
            "Start Time",
            "End Time",
            "Duration",
            "Project",
            "Category",
            "Is Productive",
            "App Name",
            "Window Title",
            "Description",
        ])?;

        for activity in activities {
            let project_name = if let Some(project_id) = activity.project_id {
                self.storage.get_project(project_id).await
                    .map(|p| p.name)
                    .unwrap_or_default()
            } else {
                String::new()
            };

            wtr.write_record(&[
                activity.id.map(|id| id.to_string()).unwrap_or_default(),
                activity.name.clone(),
                activity.start_time.to_rfc3339(),
                activity.end_time.map(|t| t.to_rfc3339()).unwrap_or_default(),
                Self::format_duration(activity.duration),
                project_name,
                activity.category.clone(),
                if activity.is_productive { "Yes" } else { "No" }.to_string(),
                activity.app_name.clone(),
                activity.window_title.clone(),
                activity.description.clone().unwrap_or_default(),
            ])?;
        }

        Ok(wtr.into_inner()?)
    }

    async fn export_pomodoros_to_csv(&self, sessions: &[PomodoroSession]) -> AppResult<Vec<u8>> {
        let mut wtr = csv::Writer::from_writer(Vec::new());
        
        wtr.write_record(&[
            "ID",
            "Start Time",
            "End Time",
            "Duration",
            "Status",
            "Project",
            "Notes",
        ])?;

        for session in sessions {
            let project_name = if let Some(project_id) = session.project_id {
                self.storage.get_project(project_id).await
                    .map(|p| p.name)
                    .unwrap_or_default()
            } else {
                String::new()
            };

            wtr.write_record(&[
                session.id.map(|id| id.to_string()).unwrap_or_default(),
                session.start_time.to_rfc3339(),
                session.end_time.map(|t| t.to_rfc3339()).unwrap_or_default(),
                Self::format_duration(session.duration),
                format!("{:?}", session.status),
                project_name,
                session.notes.clone().unwrap_or_default(),
            ])?;
        }

        Ok(wtr.into_inner()?)
    }

    async fn export_to_json<T: serde::Serialize>(&self, data: &T) -> AppResult<Vec<u8>> {
        Ok(serde_json::to_vec_pretty(data)?)
    }
}

#[async_trait::async_trait]
impl ExportService for ExportManager {
    async fn export_activities(&self, start: DateTime<Local>, end: DateTime<Local>, format: ExportFormat) -> AppResult<Vec<u8>> {
        let activities = self.storage.get_activities(start, end).await?;
        
        match format {
            ExportFormat::CSV => self.export_activities_to_csv(&activities).await,
            ExportFormat::JSON => self.export_to_json(&activities).await,
            ExportFormat::Excel => Err(crate::core::error::AppError::NotImplemented("Excel export not implemented yet".into())),
        }
    }

    async fn export_pomodoros(&self, start: DateTime<Local>, end: DateTime<Local>, format: ExportFormat) -> AppResult<Vec<u8>> {
        let sessions = self.storage.get_pomodoro_sessions(start, end).await?;
        
        match format {
            ExportFormat::CSV => self.export_pomodoros_to_csv(&sessions).await,
            ExportFormat::JSON => self.export_to_json(&sessions).await,
            ExportFormat::Excel => Err(crate::core::error::AppError::NotImplemented("Excel export not implemented yet".into())),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_export_manager() {
        // TODO: 添加测试用例
    }
} 