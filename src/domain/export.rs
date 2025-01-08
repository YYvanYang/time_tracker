use crate::core::{AppResult, models::*};
use crate::core::traits::Storage;
use chrono::{DateTime, Local};
use std::sync::Arc;
use serde_json;
use csv;

pub struct ExportManager {
    storage: Arc<dyn Storage + Send + Sync>,
}

#[derive(serde::Serialize)]
struct ActivityExport {
    id: Option<i64>,
    app_name: String,
    window_title: String,
    start_time: String,
    duration_seconds: u64,
    category: Option<String>,
    is_productive: bool,
    project_id: Option<i64>,
}

#[derive(serde::Serialize)]
struct PomodoroSessionExport {
    id: Option<i64>,
    start_time: String,
    duration_seconds: u64,
    status: String,
    project_id: Option<i64>,
    notes: Option<String>,
}

impl ExportManager {
    pub fn new(storage: Arc<dyn Storage + Send + Sync>) -> Self {
        Self { storage }
    }

    pub async fn export_activities_to_json(
        &self,
        start: DateTime<Local>,
        end: DateTime<Local>,
    ) -> AppResult<String> {
        let activities = self.storage.get_activities(start, end).await?;
        
        let exports: Vec<ActivityExport> = activities.into_iter()
            .map(|a| ActivityExport {
                id: a.id,
                app_name: a.app_name,
                window_title: a.window_title,
                start_time: a.start_time.to_rfc3339(),
                duration_seconds: a.duration.as_secs(),
                category: a.category,
                is_productive: a.is_productive,
                project_id: a.project_id,
            })
            .collect();

        Ok(serde_json::to_string_pretty(&exports)?)
    }

    pub async fn export_activities_to_csv(
        &self,
        start: DateTime<Local>,
        end: DateTime<Local>,
    ) -> AppResult<String> {
        let activities = self.storage.get_activities(start, end).await?;
        let mut wtr = csv::Writer::from_writer(vec![]);

        for activity in activities {
            wtr.serialize(ActivityExport {
                id: activity.id,
                app_name: activity.app_name,
                window_title: activity.window_title,
                start_time: activity.start_time.to_rfc3339(),
                duration_seconds: activity.duration.as_secs(),
                category: activity.category,
                is_productive: activity.is_productive,
                project_id: activity.project_id,
            })?;
        }

        let data = wtr.into_inner().map_err(|e| crate::core::AppError::CsvWriter(e))?;
        String::from_utf8(data).map_err(|e| e.into())
    }

    pub async fn export_pomodoros_to_json(
        &self,
        start: DateTime<Local>,
        end: DateTime<Local>,
    ) -> AppResult<String> {
        let sessions = self.storage.get_pomodoro_sessions(start, end).await?;
        
        let exports: Vec<PomodoroSessionExport> = sessions.into_iter()
            .map(|s| PomodoroSessionExport {
                id: s.id,
                start_time: s.start_time.to_rfc3339(),
                duration_seconds: s.duration.as_secs(),
                status: format!("{:?}", s.status),
                project_id: s.project_id,
                notes: s.notes,
            })
            .collect();

        Ok(serde_json::to_string_pretty(&exports)?)
    }

    pub async fn export_pomodoros_to_csv(
        &self,
        start: DateTime<Local>,
        end: DateTime<Local>,
    ) -> AppResult<String> {
        let sessions = self.storage.get_pomodoro_sessions(start, end).await?;
        let mut wtr = csv::Writer::from_writer(vec![]);

        for session in sessions {
            wtr.serialize(PomodoroSessionExport {
                id: session.id,
                start_time: session.start_time.to_rfc3339(),
                duration_seconds: session.duration.as_secs(),
                status: format!("{:?}", session.status),
                project_id: session.project_id,
                notes: session.notes,
            })?;
        }

        let data = wtr.into_inner().map_err(|e| crate::core::AppError::CsvWriter(e))?;
        String::from_utf8(data).map_err(|e| e.into())
    }

    pub async fn export_project_data_to_json(
        &self,
        project_id: i64,
        start: DateTime<Local>,
        end: DateTime<Local>,
    ) -> AppResult<String> {
        let activities = self.storage.get_project_activities(project_id, start, end).await?;
        let sessions = self.storage.get_project_pomodoro_sessions(project_id, start, end).await?;

        let export = ProjectExport {
            activities: activities.into_iter()
                .map(|a| ActivityExport {
                    id: a.id,
                    app_name: a.app_name,
                    window_title: a.window_title,
                    start_time: a.start_time.to_rfc3339(),
                    duration_seconds: a.duration.as_secs(),
                    category: a.category,
                    is_productive: a.is_productive,
                    project_id: a.project_id,
                })
                .collect(),
            pomodoros: sessions.into_iter()
                .map(|s| PomodoroSessionExport {
                    id: s.id,
                    start_time: s.start_time.to_rfc3339(),
                    duration_seconds: s.duration.as_secs(),
                    status: format!("{:?}", s.status),
                    project_id: s.project_id,
                    notes: s.notes,
                })
                .collect(),
        };

        Ok(serde_json::to_string_pretty(&export)?)
    }
}

#[derive(serde::Serialize)]
struct ProjectExport {
    activities: Vec<ActivityExport>,
    pomodoros: Vec<PomodoroSessionExport>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::models::*;
    use std::time::Duration;

    #[tokio::test]
    async fn test_export_activities() -> AppResult<()> {
        // TODO: 实现测试
        Ok(())
    }
} 