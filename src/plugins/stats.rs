use crate::core::{AppResult, models::*};
use crate::domain::plugin::{Plugin, PluginMetadata};
use async_trait::async_trait;
use chrono::{DateTime, Local, Duration as ChronoDuration, Datelike, Timelike};
use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use std::time::Duration;
use tokio::sync::RwLock;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatsConfig {
    pub auto_analyze: bool,
    pub analyze_interval: Duration,
    pub productivity_threshold: f64,
    pub work_hours: WorkHours,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkHours {
    pub start_hour: u32,
    pub end_hour: u32,
    pub work_days: Vec<u32>, // 1 = Monday, 7 = Sunday
}

impl Default for StatsConfig {
    fn default() -> Self {
        Self {
            auto_analyze: true,
            analyze_interval: Duration::from_secs(60 * 60), // 1 hour
            productivity_threshold: 0.7,
            work_hours: WorkHours {
                start_hour: 9,
                end_hour: 18,
                work_days: vec![1, 2, 3, 4, 5], // Monday to Friday
            },
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProductivityStats {
    pub period_start: DateTime<Local>,
    pub period_end: DateTime<Local>,
    pub total_time: Duration,
    pub productive_time: Duration,
    pub productivity_score: f64,
    pub activity_breakdown: HashMap<String, Duration>,
    pub hourly_productivity: HashMap<u32, f64>,
    pub completed_pomodoros: usize,
}

pub struct StatsPlugin {
    metadata: PluginMetadata,
    config: RwLock<StatsConfig>,
    last_analysis: RwLock<Option<DateTime<Local>>>,
    current_stats: RwLock<Option<ProductivityStats>>,
}

impl StatsPlugin {
    pub fn new() -> Self {
        Self {
            metadata: PluginMetadata {
                id: "stats".into(),
                name: "Statistics Plugin".into(),
                version: "1.0.0".into(),
                author: "Time Tracker".into(),
                description: "Analyze productivity and time usage patterns".into(),
                dependencies: vec![],
                config_schema: Some(serde_json::to_value(StatsConfig::default()).unwrap()),
            },
            config: RwLock::new(StatsConfig::default()),
            last_analysis: RwLock::new(None),
            current_stats: RwLock::new(None),
        }
    }

    async fn analyze_productivity(&self, activities: &[Activity], pomodoros: &[PomodoroSession]) -> ProductivityStats {
        let now = Local::now();
        let period_start = now - ChronoDuration::days(1);
        let mut stats = ProductivityStats {
            period_start,
            period_end: now,
            total_time: Duration::from_secs(0),
            productive_time: Duration::from_secs(0),
            productivity_score: 0.0,
            activity_breakdown: HashMap::new(),
            hourly_productivity: HashMap::new(),
            completed_pomodoros: 0,
        };

        // 计算总时间和生产时间
        for activity in activities {
            stats.total_time += activity.duration;
            if activity.is_productive {
                stats.productive_time += activity.duration;
            }

            // 更新活动分类统计
            let category = activity.category.clone().unwrap_or_else(|| "未分类".to_string());
            *stats.activity_breakdown.entry(category).or_insert(Duration::from_secs(0)) += activity.duration;

            // 更新小时生产力统计
            let hour = activity.start_time.hour();
            let productivity = if activity.is_productive { 1.0 } else { 0.0 };
            let entry = stats.hourly_productivity.entry(hour).or_insert(0.0);
            *entry = (*entry + productivity) / 2.0;
        }

        // 计算生产力得分
        if stats.total_time.as_secs() > 0 {
            stats.productivity_score = stats.productive_time.as_secs_f64() / stats.total_time.as_secs_f64();
        }

        // 统计完成的番茄钟数量
        stats.completed_pomodoros = pomodoros.iter()
            .filter(|p| matches!(p.status, PomodoroStatus::Completed))
            .count();

        stats
    }

    async fn check_work_hours(&self, time: DateTime<Local>) -> bool {
        let config = self.config.read().await;
        let work_hours = &config.work_hours;

        // 检查是否是工作日
        let weekday = time.weekday().number_from_monday();
        if !work_hours.work_days.contains(&weekday) {
            return false;
        }

        // 检查是否在工作时间内
        let hour = time.hour();
        hour >= work_hours.start_hour && hour < work_hours.end_hour
    }

    async fn should_analyze(&self) -> bool {
        let config = self.config.read().await;
        if !config.auto_analyze {
            return false;
        }

        if let Some(last_analysis) = *self.last_analysis.read().await {
            let elapsed = Local::now().signed_duration_since(last_analysis);
            elapsed.to_std().map(|d| d >= config.analyze_interval).unwrap_or(false)
        } else {
            true
        }
    }

    pub async fn get_current_stats(&self) -> Option<ProductivityStats> {
        self.current_stats.read().await.clone()
    }

    pub async fn get_productivity_alerts(&self) -> Vec<String> {
        let mut alerts = Vec::new();
        
        if let Some(stats) = self.current_stats.read().await.as_ref() {
            let config = self.config.read().await;
            
            // 检查生产力得分
            if stats.productivity_score < config.productivity_threshold {
                alerts.push(format!(
                    "生产力得分较低 ({:.1}%), 建议关注工作效率",
                    stats.productivity_score * 100.0
                ));
            }

            // 检查工作时间分布
            let work_time_ratio = stats.productive_time.as_secs_f64() / 
                (ChronoDuration::hours(8).num_seconds() as f64);
            if work_time_ratio < 0.5 {
                alerts.push("工作时间不足,建议增加有效工作时间".into());
            }

            // 检查番茄钟完成情况
            if stats.completed_pomodoros < 4 {
                alerts.push("番茄钟完成数量较少,建议使用番茄工作法提高专注度".into());
            }
        }

        alerts
    }
}

#[async_trait]
impl Plugin for StatsPlugin {
    fn metadata(&self) -> &PluginMetadata {
        &self.metadata
    }

    async fn initialize(&self, config: Option<serde_json::Value>) -> AppResult<()> {
        if let Some(config) = config {
            let stats_config: StatsConfig = serde_json::from_value(config)?;
            *self.config.write().await = stats_config;
        }
        Ok(())
    }

    async fn start(&self) -> AppResult<()> {
        Ok(())
    }

    async fn stop(&self) -> AppResult<()> {
        Ok(())
    }

    async fn on_activity_change(&self, activity: &Activity) -> AppResult<()> {
        if self.should_analyze().await && self.check_work_hours(activity.start_time).await {
            let activities = vec![activity.clone()]; // TODO: 从存储中获取活动数据
            let pomodoros = vec![]; // TODO: 从存储中获取番茄钟数据
            let stats = self.analyze_productivity(&activities, &pomodoros).await;
            *self.current_stats.write().await = Some(stats);
            *self.last_analysis.write().await = Some(Local::now());
        }
        Ok(())
    }

    async fn on_pomodoro_start(&self, _session: &PomodoroSession) -> AppResult<()> {
        Ok(())
    }

    async fn on_pomodoro_end(&self, session: &PomodoroSession) -> AppResult<()> {
        if self.should_analyze().await && self.check_work_hours(session.start_time).await {
            let activities = vec![]; // TODO: 从存储中获取活动数据
            let pomodoros = vec![session.clone()];
            let stats = self.analyze_productivity(&activities, &pomodoros).await;
            *self.current_stats.write().await = Some(stats);
            *self.last_analysis.write().await = Some(Local::now());
        }
        Ok(())
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
    use std::time::Duration;

    #[tokio::test]
    async fn test_stats_analysis() -> AppResult<()> {
        let plugin = StatsPlugin::new();
        let now = Local::now();

        // 创建测试数据
        let activities = vec![
            Activity {
                id: Some(1),
                app_name: "code_editor".into(),
                window_title: "project.rs".into(),
                start_time: now,
                duration: Duration::from_secs(3600),
                category: Some("coding".into()),
                is_productive: true,
                project_id: None,
            },
            Activity {
                id: Some(2),
                app_name: "browser".into(),
                window_title: "social_media".into(),
                start_time: now + ChronoDuration::hours(1),
                duration: Duration::from_secs(1800),
                category: Some("entertainment".into()),
                is_productive: false,
                project_id: None,
            },
        ];

        let pomodoros = vec![
            PomodoroSession {
                id: Some(1),
                start_time: now,
                duration: Duration::from_secs(1500),
                status: PomodoroStatus::Completed,
                project_id: None,
                notes: None,
            },
        ];

        // 分析数据
        let stats = plugin.analyze_productivity(&activities, &pomodoros).await;

        // 验证结果
        assert_eq!(stats.total_time, Duration::from_secs(5400));
        assert_eq!(stats.productive_time, Duration::from_secs(3600));
        assert!(stats.productivity_score > 0.0);
        assert_eq!(stats.completed_pomodoros, 1);
        assert_eq!(stats.activity_breakdown.len(), 2);

        Ok(())
    }
} 