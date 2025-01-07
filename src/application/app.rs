use crate::core::AppResult;
use crate::infrastructure::config::Config;
use crate::plugins::PluginRegistry;
use crate::application::services::ServiceContainer;
use crate::application::events::{AppEvent, EventBus};
use crate::application::commands::CommandHandler;
use crate::application::queries::QueryHandler;
use std::sync::Arc;
use tokio::sync::broadcast;
use tokio::task::JoinHandle;

pub struct App {
    services: Arc<ServiceContainer>,
    event_bus: EventBus,
    command_handler: Arc<CommandHandler>,
    query_handler: Arc<QueryHandler>,
    plugin_registry: Arc<PluginRegistry>,
    background_tasks: Vec<JoinHandle<()>>,
}

impl App {
    pub async fn new(config: Config, plugin_registry: Arc<PluginRegistry>) -> AppResult<Self> {
        // 初始化服务容器
        let services = Arc::new(ServiceContainer::new(config));
        
        // 初始化事件总线
        let event_bus = EventBus::default();
        
        // 初始化命令处理器和查询处理器
        let command_handler = Arc::new(CommandHandler::new(
            services.clone(),
            event_bus.clone(),
            plugin_registry.clone(),
        ));
        
        let query_handler = Arc::new(QueryHandler::new(
            services.clone(),
            plugin_registry.clone(),
        ));

        Ok(Self {
            services,
            event_bus,
            command_handler,
            query_handler,
            plugin_registry,
            background_tasks: Vec::new(),
        })
    }

    pub fn command_handler(&self) -> Arc<CommandHandler> {
        self.command_handler.clone()
    }

    pub fn query_handler(&self) -> Arc<QueryHandler> {
        self.query_handler.clone()
    }

    pub fn subscribe_events(&self) -> broadcast::Receiver<AppEvent> {
        self.event_bus.subscribe()
    }

    pub async fn start(&mut self) -> AppResult<()> {
        // 发布应用启动事件
        self.event_bus.publish(AppEvent::ApplicationStarted);
        
        // 启动所有插件
        self.plugin_registry.start_all().await?;
        
        // TODO: 启动其他后台任务
        
        Ok(())
    }

    pub async fn stop(&mut self) -> AppResult<()> {
        // 发布应用停止事件
        self.event_bus.publish(AppEvent::ApplicationStopping);
        
        // 停止所有插件
        self.plugin_registry.stop_all().await?;
        
        // 停止所有后台任务
        for task in self.background_tasks.drain(..) {
            task.abort();
        }
        
        Ok(())
    }
} 