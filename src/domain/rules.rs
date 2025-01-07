use crate::core::{AppResult, models::*};
use crate::core::traits::Storage;
use chrono::{DateTime, Local};
use std::sync::Arc;
use tokio::sync::RwLock;
use regex::Regex;
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Rule {
    pub id: Option<i64>,
    pub name: String,
    pub app_pattern: Option<String>,
    pub title_pattern: Option<String>,
    pub category: Option<String>,
    pub is_productive: bool,
    pub priority: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuleMatch {
    pub rule: Rule,
    pub activity: Activity,
    pub matched_patterns: Vec<String>,
}

pub struct RuleEngine {
    storage: Arc<dyn Storage>,
    rules: RwLock<Vec<Rule>>,
    app_patterns: RwLock<Vec<(Regex, Rule)>>,
    title_patterns: RwLock<Vec<(Regex, Rule)>>,
}

impl RuleEngine {
    pub fn new(storage: Arc<dyn Storage>) -> Self {
        Self {
            storage,
            rules: RwLock::new(Vec::new()),
            app_patterns: RwLock::new(Vec::new()),
            title_patterns: RwLock::new(Vec::new()),
        }
    }

    pub async fn load_rules(&self) -> AppResult<()> {
        let rules = self.storage.get_rules().await?;
        
        let mut app_patterns = Vec::new();
        let mut title_patterns = Vec::new();

        for rule in &rules {
            if let Some(pattern) = &rule.app_pattern {
                if let Ok(regex) = Regex::new(pattern) {
                    app_patterns.push((regex, rule.clone()));
                }
            }
            if let Some(pattern) = &rule.title_pattern {
                if let Ok(regex) = Regex::new(pattern) {
                    title_patterns.push((regex, rule.clone()));
                }
            }
        }

        // 按优先级排序
        app_patterns.sort_by(|a, b| b.1.priority.cmp(&a.1.priority));
        title_patterns.sort_by(|a, b| b.1.priority.cmp(&a.1.priority));

        *self.rules.write().await = rules;
        *self.app_patterns.write().await = app_patterns;
        *self.title_patterns.write().await = title_patterns;

        Ok(())
    }

    pub async fn add_rule(&self, rule: Rule) -> AppResult<Rule> {
        let rule = self.storage.save_rule(&rule).await?;
        self.load_rules().await?;
        Ok(rule)
    }

    pub async fn update_rule(&self, rule: Rule) -> AppResult<()> {
        self.storage.save_rule(&rule).await?;
        self.load_rules().await?;
        Ok(())
    }

    pub async fn delete_rule(&self, id: i64) -> AppResult<()> {
        self.storage.delete_rule(id).await?;
        self.load_rules().await?;
        Ok(())
    }

    pub async fn get_rules(&self) -> Vec<Rule> {
        self.rules.read().await.clone()
    }

    pub async fn classify_activity(&self, activity: &Activity) -> Option<RuleMatch> {
        let mut matches = Vec::new();

        // 检查应用名称匹配
        for (pattern, rule) in self.app_patterns.read().await.iter() {
            if pattern.is_match(&activity.app_name) {
                matches.push(RuleMatch {
                    rule: rule.clone(),
                    activity: activity.clone(),
                    matched_patterns: vec![format!("app: {}", &activity.app_name)],
                });
            }
        }

        // 检查窗口标题匹配
        for (pattern, rule) in self.title_patterns.read().await.iter() {
            if pattern.is_match(&activity.window_title) {
                matches.push(RuleMatch {
                    rule: rule.clone(),
                    activity: activity.clone(),
                    matched_patterns: vec![format!("title: {}", &activity.window_title)],
                });
            }
        }

        // 返回优先级最高的匹配
        matches.into_iter()
            .max_by_key(|m| m.rule.priority)
    }

    pub async fn apply_rules(&self, activity: &mut Activity) -> AppResult<()> {
        if let Some(rule_match) = self.classify_activity(activity).await {
            activity.category = rule_match.rule.category;
            activity.is_productive = rule_match.rule.is_productive;
        }
        Ok(())
    }

    pub async fn analyze_rules(&self, start: DateTime<Local>, end: DateTime<Local>) -> AppResult<Vec<RuleMatch>> {
        let activities = self.storage.get_activities(start, end).await?;
        let mut matches = Vec::new();

        for activity in activities {
            if let Some(rule_match) = self.classify_activity(&activity).await {
                matches.push(rule_match);
            }
        }

        Ok(matches)
    }

    pub async fn suggest_rules(&self, start: DateTime<Local>, end: DateTime<Local>) -> AppResult<Vec<Rule>> {
        let activities = self.storage.get_activities(start, end).await?;
        let mut suggestions = Vec::new();
        let mut app_frequencies = std::collections::HashMap::new();

        // 统计应用使用频率
        for activity in activities {
            let entry = app_frequencies
                .entry(activity.app_name)
                .or_insert((0, activity.is_productive));
            entry.0 += 1;
        }

        // 为高频应用生成规则建议
        for (app_name, (frequency, is_productive)) in app_frequencies {
            if frequency >= 10 {  // 阈值可配置
                suggestions.push(Rule {
                    id: None,
                    name: format!("Suggested rule for {}", app_name),
                    app_pattern: Some(regex::escape(&app_name)),
                    title_pattern: None,
                    category: None,
                    is_productive,
                    priority: 0,
                });
            }
        }

        Ok(suggestions)
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
            async fn get_rules(&self) -> AppResult<Vec<Rule>>;
            async fn save_rule(&self, rule: &Rule) -> AppResult<Rule>;
            async fn delete_rule(&self, id: i64) -> AppResult<()>;
            async fn get_activities(&self, start: DateTime<Local>, end: DateTime<Local>) -> AppResult<Vec<Activity>>;
        }
    }

    #[tokio::test]
    async fn test_rule_engine() -> AppResult<()> {
        let mut mock_storage = MockStorage::new();
        let now = Local::now();
        
        // 设置模拟规则数据
        mock_storage
            .expect_get_rules()
            .returning(|| Ok(vec![
                Rule {
                    id: Some(1),
                    name: "Test Rule".into(),
                    app_pattern: Some("test_app".into()),
                    title_pattern: None,
                    category: Some("work".into()),
                    is_productive: true,
                    priority: 1,
                }
            ]));

        let engine = RuleEngine::new(Arc::new(mock_storage));
        engine.load_rules().await?;

        // 测试规则匹配
        let mut activity = Activity {
            id: Some(1),
            app_name: "test_app".into(),
            window_title: "test_window".into(),
            start_time: now,
            duration: Duration::from_secs(3600),
            category: None,
            is_productive: false,
            project_id: None,
        };

        engine.apply_rules(&mut activity).await?;
        assert_eq!(activity.category, Some("work".into()));
        assert!(activity.is_productive);

        Ok(())
    }
} 