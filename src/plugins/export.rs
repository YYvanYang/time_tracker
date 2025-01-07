use crate::core::{AppResult, models::*};
use crate::domain::plugin::{Plugin, PluginMetadata};
use async_trait::async_trait;
use chrono::{DateTime, Local, Duration as ChronoDuration};
use serde::{Serialize, Deserialize};
use std::path::{Path, PathBuf};
use std::time::Duration;
use tokio::fs;
use tokio::sync::RwLock;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportConfig {
    pub export_dir: String,
    pub auto_export: bool,
    pub export_interval: Duration,
    pub formats: Vec<ExportFormat>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ExportFormat {
    JSON,
    CSV,
    Excel,
    Markdown,
}

impl Default for ExportConfig {
    fn default() -> Self {
        Self {
            export_dir: "exports".into(),
            auto_export: true,
            export_interval: Duration::from_secs(24 * 60 * 60), // 1 day
            formats: vec![ExportFormat::JSON, ExportFormat::CSV],
        }
    }
}

pub struct ExportPlugin {
    metadata: PluginMetadata,
    config: RwLock<ExportConfig>,
    last_export: RwLock<Option<DateTime<Local>>>,
}

impl ExportPlugin {
    pub fn new() -> Self {
        Self {
            metadata: PluginMetadata {
                id: "export".into(),
                name: "Export Plugin".into(),
                version: "1.0.0".into(),
                author: "Time Tracker".into(),
                description: "Export time tracking data in various formats".into(),
                dependencies: vec![],
                config_schema: Some(serde_json::to_value(ExportConfig::default()).unwrap()),
            },
            config: RwLock::new(ExportConfig::default()),
            last_export: RwLock::new(None),
        }
    }

    async fn export_data(&self) -> AppResult<Vec<PathBuf>> {
        let config = self.config.read().await;
        let export_dir = Path::new(&config.export_dir);

        // 确保导出目录存在
        if !export_dir.exists() {
            fs::create_dir_all(export_dir).await?;
        }

        let now = Local::now();
        let start = now - ChronoDuration::days(7); // 导出最近一周的数据
        let timestamp = now.format("%Y%m%d_%H%M%S");
        let mut exported_files = Vec::new();

        // 准备导出数据
        let activities = vec![]; // TODO: 从存储中获取活动数据
        let pomodoros = vec![]; // TODO: 从存储中获取番茄钟数据

        for format in &config.formats {
            match format {
                ExportFormat::JSON => {
                    let file_path = export_dir.join(format!("activities_{}.json", timestamp));
                    let json = serde_json::to_string_pretty(&activities)?;
                    fs::write(&file_path, json).await?;
                    exported_files.push(file_path);

                    let file_path = export_dir.join(format!("pomodoros_{}.json", timestamp));
                    let json = serde_json::to_string_pretty(&pomodoros)?;
                    fs::write(&file_path, json).await?;
                    exported_files.push(file_path);
                }
                ExportFormat::CSV => {
                    let file_path = export_dir.join(format!("activities_{}.csv", timestamp));
                    let mut wtr = csv::Writer::from_writer(vec![]);
                    for activity in &activities {
                        wtr.serialize(activity)?;
                    }
                    fs::write(&file_path, String::from_utf8(wtr.into_inner())?).await?;
                    exported_files.push(file_path);

                    let file_path = export_dir.join(format!("pomodoros_{}.csv", timestamp));
                    let mut wtr = csv::Writer::from_writer(vec![]);
                    for pomodoro in &pomodoros {
                        wtr.serialize(pomodoro)?;
                    }
                    fs::write(&file_path, String::from_utf8(wtr.into_inner())?).await?;
                    exported_files.push(file_path);
                }
                ExportFormat::Excel => {
                    let file_path = export_dir.join(format!("timetracker_{}.xlsx", timestamp));
                    // TODO: 实现 Excel 导出
                    exported_files.push(file_path);
                }
                ExportFormat::Markdown => {
                    let file_path = export_dir.join(format!("timetracker_{}.md", timestamp));
                    let mut content = String::new();
                    
                    // 添加活动报告
                    content.push_str("# Activity Report\n\n");
                    content.push_str("## Activities\n\n");
                    content.push_str("| Time | Application | Window | Duration | Category | Productive |\n");
                    content.push_str("|------|-------------|---------|-----------|-----------|------------|\n");
                    for activity in &activities {
                        content.push_str(&format!(
                            "| {} | {} | {} | {:?} | {} | {} |\n",
                            activity.start_time.format("%Y-%m-%d %H:%M:%S"),
                            activity.app_name,
                            activity.window_title,
                            activity.duration,
                            activity.category.as_deref().unwrap_or("N/A"),
                            activity.is_productive
                        ));
                    }

                    // 添加番茄钟报告
                    content.push_str("\n## Pomodoro Sessions\n\n");
                    content.push_str("| Time | Duration | Status | Notes |\n");
                    content.push_str("|------|-----------|--------|--------|\n");
                    for pomodoro in &pomodoros {
                        content.push_str(&format!(
                            "| {} | {:?} | {:?} | {} |\n",
                            pomodoro.start_time.format("%Y-%m-%d %H:%M:%S"),
                            pomodoro.duration,
                            pomodoro.status,
                            pomodoro.notes.as_deref().unwrap_or("N/A")
                        ));
                    }

                    fs::write(&file_path, content).await?;
                    exported_files.push(file_path);
                }
            }
        }

        // 更新最后导出时间
        *self.last_export.write().await = Some(now);

        Ok(exported_files)
    }

    async fn check_auto_export(&self) -> AppResult<()> {
        let config = self.config.read().await;
        if !config.auto_export {
            return Ok(());
        }

        if let Some(last_export) = *self.last_export.read().await {
            let elapsed = Local::now().signed_duration_since(last_export);
            if elapsed.to_std()? >= config.export_interval {
                self.export_data().await?;
            }
        } else {
            self.export_data().await?;
        }

        Ok(())
    }
}

#[async_trait]
impl Plugin for ExportPlugin {
    fn metadata(&self) -> &PluginMetadata {
        &self.metadata
    }

    async fn initialize(&self, config: Option<serde_json::Value>) -> AppResult<()> {
        if let Some(config) = config {
            let export_config: ExportConfig = serde_json::from_value(config)?;
            *self.config.write().await = export_config;
        }
        Ok(())
    }

    async fn start(&self) -> AppResult<()> {
        self.check_auto_export().await
    }

    async fn stop(&self) -> AppResult<()> {
        Ok(())
    }

    async fn on_activity_change(&self, _activity: &Activity) -> AppResult<()> {
        Ok(())
    }

    async fn on_pomodoro_start(&self, _session: &PomodoroSession) -> AppResult<()> {
        Ok(())
    }

    async fn on_pomodoro_end(&self, _session: &PomodoroSession) -> AppResult<()> {
        self.check_auto_export().await
    }

    async fn on_break_start(&self, _duration: Duration) -> AppResult<()> {
        Ok(())
    }

    async fn on_break_end(&self) -> AppResult<()> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_export_lifecycle() -> AppResult<()> {
        // 创建临时目录
        let temp_dir = tempdir()?;
        let export_dir = temp_dir.path().join("exports");

        // 创建插件实例
        let plugin = ExportPlugin::new();
        
        // 配置插件
        let config = ExportConfig {
            export_dir: export_dir.to_string_lossy().into(),
            auto_export: true,
            export_interval: Duration::from_secs(1),
            formats: vec![ExportFormat::JSON, ExportFormat::CSV, ExportFormat::Markdown],
        };
        plugin.initialize(Some(serde_json::to_value(config)?)).await?;

        // 执行导出
        let exported_files = plugin.export_data().await?;
        
        // 验证导出的文件
        assert!(!exported_files.is_empty());
        for file in &exported_files {
            assert!(file.exists());
        }

        // 清理
        temp_dir.close()?;

        Ok(())
    }
} 