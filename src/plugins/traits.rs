use crate::core::AppResult;
use async_trait::async_trait;
use std::any::Any;

/// 插件接口
#[async_trait]
pub trait Plugin: Send + Sync {
    /// 获取插件名称
    fn name(&self) -> &str;
    
    /// 获取插件版本
    fn version(&self) -> &str;
    
    /// 获取插件描述
    fn description(&self) -> &str;
    
    /// 初始化插件
    async fn initialize(&self) -> AppResult<()>;
    
    /// 启动插件
    async fn start(&self) -> AppResult<()>;
    
    /// 停止插件
    async fn stop(&self) -> AppResult<()>;
    
    /// 卸载插件
    async fn uninstall(&self) -> AppResult<()>;
    
    /// 获取插件配置界面
    fn get_settings_ui(&self) -> Option<Box<dyn Any>>;
}

/// 活动插件接口
#[async_trait]
pub trait ActivityPlugin: Plugin {
    /// 活动开始时调用
    async fn on_activity_start(&self, activity: &crate::core::models::Activity) -> AppResult<()>;
    
    /// 活动结束时调用
    async fn on_activity_end(&self, activity: &crate::core::models::Activity) -> AppResult<()>;
    
    /// 活动更新时调用
    async fn on_activity_update(&self, activity: &crate::core::models::Activity) -> AppResult<()>;
}

/// 番茄钟插件接口
#[async_trait]
pub trait PomodoroPlugin: Plugin {
    /// 番茄钟开始时调用
    async fn on_pomodoro_start(&self, session: &crate::core::models::PomodoroSession) -> AppResult<()>;
    
    /// 番茄钟暂停时调用
    async fn on_pomodoro_pause(&self, session: &crate::core::models::PomodoroSession) -> AppResult<()>;
    
    /// 番茄钟恢复时调用
    async fn on_pomodoro_resume(&self, session: &crate::core::models::PomodoroSession) -> AppResult<()>;
    
    /// 番茄钟完成时调用
    async fn on_pomodoro_complete(&self, session: &crate::core::models::PomodoroSession) -> AppResult<()>;
    
    /// 番茄钟中断时调用
    async fn on_pomodoro_interrupt(&self, session: &crate::core::models::PomodoroSession) -> AppResult<()>;
}

/// 统计插件接口
#[async_trait]
pub trait StatisticsPlugin: Plugin {
    /// 生成统计报告
    async fn generate_report(&self, start: chrono::DateTime<chrono::Local>, end: chrono::DateTime<chrono::Local>) -> AppResult<String>;
    
    /// 导出数据
    async fn export_data(&self, start: chrono::DateTime<chrono::Local>, end: chrono::DateTime<chrono::Local>) -> AppResult<Vec<u8>>;
}

/// 通知插件接口
#[async_trait]
pub trait NotificationPlugin: Plugin {
    /// 发送通知
    async fn send_notification(&self, title: &str, message: &str) -> AppResult<()>;
}

/// 备份插件接口
#[async_trait]
pub trait BackupPlugin: Plugin {
    /// 创建备份
    async fn create_backup(&self) -> AppResult<()>;
    
    /// 恢复备份
    async fn restore_backup(&self, backup_id: &str) -> AppResult<()>;
    
    /// 列出所有备份
    async fn list_backups(&self) -> AppResult<Vec<String>>;
    
    /// 删除备份
    async fn delete_backup(&self, backup_id: &str) -> AppResult<()>;
} 