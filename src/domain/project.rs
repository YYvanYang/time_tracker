use crate::core::{AppResult, models::*, traits::*};
use crate::core::traits::Storage;
use chrono::{DateTime, Local};
use std::sync::Arc;

pub struct ProjectManager {
    storage: Arc<dyn Storage + Send + Sync>,
}

impl ProjectManager {
    pub fn new(storage: Arc<dyn Storage + Send + Sync>) -> Self {
        Self { storage }
    }
}

#[async_trait::async_trait]
impl ProjectService for ProjectManager {
    async fn create_project(&self, project: Project) -> AppResult<i64> {
        self.storage.save_project(&project).await
    }

    async fn update_project(&self, project: Project) -> AppResult<()> {
        self.storage.save_project(&project).await?;
        Ok(())
    }

    async fn delete_project(&self, id: i64) -> AppResult<()> {
        // TODO: 实现删除项目的功能
        Ok(())
    }

    async fn get_project(&self, id: i64) -> AppResult<Project> {
        self.storage.get_project(id).await
    }

    async fn list_projects(&self) -> AppResult<Vec<Project>> {
        self.storage.list_projects().await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_project_manager() {
        // TODO: 添加测试用例
    }
} 