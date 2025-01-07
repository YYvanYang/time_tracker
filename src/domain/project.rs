use crate::core::{AppResult, models::*};
use crate::core::traits::Storage;
use chrono::{DateTime, Local};
use std::sync::Arc;

pub struct ProjectManager {
    storage: Arc<dyn Storage>,
}

impl ProjectManager {
    pub fn new(storage: Arc<dyn Storage>) -> Self {
        Self { storage }
    }

    pub async fn create_project(&self, name: String, description: Option<String>) -> AppResult<Project> {
        let project = Project {
            id: None,
            name,
            description,
            created_at: Local::now(),
            updated_at: Local::now(),
            is_archived: false,
        };
        
        let id = self.storage.save_project(&project).await?;
        Ok(Project { id: Some(id), ..project })
    }

    pub async fn update_project(&self, project: Project) -> AppResult<()> {
        let mut updated = project;
        updated.updated_at = Local::now();
        self.storage.save_project(&updated).await?;
        Ok(())
    }

    pub async fn delete_project(&self, id: i64) -> AppResult<()> {
        self.storage.delete_project(id).await
    }

    pub async fn get_project(&self, id: i64) -> AppResult<Option<Project>> {
        self.storage.get_project(id).await
    }

    pub async fn get_all_projects(&self) -> AppResult<Vec<Project>> {
        self.storage.get_all_projects().await
    }

    pub async fn get_active_projects(&self) -> AppResult<Vec<Project>> {
        let projects = self.storage.get_all_projects().await?;
        Ok(projects.into_iter().filter(|p| !p.is_archived).collect())
    }

    pub async fn archive_project(&self, id: i64) -> AppResult<()> {
        if let Some(mut project) = self.storage.get_project(id).await? {
            project.is_archived = true;
            project.updated_at = Local::now();
            self.storage.save_project(&project).await?;
        }
        Ok(())
    }

    pub async fn unarchive_project(&self, id: i64) -> AppResult<()> {
        if let Some(mut project) = self.storage.get_project(id).await? {
            project.is_archived = false;
            project.updated_at = Local::now();
            self.storage.save_project(&project).await?;
        }
        Ok(())
    }

    pub async fn get_project_activities(
        &self,
        project_id: i64,
        start: DateTime<Local>,
        end: DateTime<Local>,
    ) -> AppResult<Vec<Activity>> {
        self.storage.get_project_activities(project_id, start, end).await
    }

    pub async fn get_project_stats(&self, project_id: i64) -> AppResult<ProjectStats> {
        self.storage.get_project_stats(project_id).await
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
            async fn save_project(&self, project: &Project) -> AppResult<i64>;
            async fn get_project(&self, id: i64) -> AppResult<Option<Project>>;
            async fn get_all_projects(&self) -> AppResult<Vec<Project>>;
            async fn delete_project(&self, id: i64) -> AppResult<()>;
            async fn get_project_activities(&self, project_id: i64, start: DateTime<Local>, end: DateTime<Local>) -> AppResult<Vec<Activity>>;
            async fn get_project_stats(&self, project_id: i64) -> AppResult<ProjectStats>;
        }
    }

    #[tokio::test]
    async fn test_project_lifecycle() -> AppResult<()> {
        let mut mock_storage = MockStorage::new();
        
        // 设置预期行为
        mock_storage
            .expect_save_project()
            .returning(|_| Ok(1));
        
        mock_storage
            .expect_get_project()
            .with(eq(1))
            .returning(|_| Ok(Some(Project {
                id: Some(1),
                name: "Test Project".into(),
                description: None,
                created_at: Local::now(),
                updated_at: Local::now(),
                is_archived: false,
            })));

        let manager = ProjectManager::new(Arc::new(mock_storage));

        // 测试创建项目
        let project = manager.create_project("Test Project".into(), None).await?;
        assert_eq!(project.name, "Test Project");
        assert!(project.id.is_some());

        // 测试获取项目
        let retrieved = manager.get_project(1).await?.unwrap();
        assert_eq!(retrieved.name, "Test Project");

        Ok(())
    }
} 