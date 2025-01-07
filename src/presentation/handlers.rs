use crate::application::App;
use crate::application::events::AppEvent;
use crate::presentation::state::SharedState;
use chrono::Local;
use std::sync::Arc;
use tokio::sync::broadcast;

pub struct EventHandler {
    app: Arc<App>,
    state: SharedState,
    rx: broadcast::Receiver<AppEvent>,
}

impl EventHandler {
    pub fn new(app: Arc<App>, state: SharedState) -> Self {
        let rx = app.subscribe_events();
        Self { app, state, rx }
    }

    pub async fn run(&mut self) {
        while let Ok(event) = self.rx.recv().await {
            self.handle_event(event).await;
        }
    }

    async fn handle_event(&self, event: AppEvent) {
        match event {
            AppEvent::ActivityStarted(activity) => {
                let mut state = self.state.write().await;
                state.set_current_activity(Some((*activity).clone()));
                
                // 更新活动列表
                if let Ok(activities) = self.app.query_handler().get_daily_activities().await {
                    state.update_activities(activities);
                }
            }
            AppEvent::ActivityStopped(activity) => {
                let mut state = self.state.write().await;
                state.set_current_activity(None);
                
                // 更新统计信息
                let now = Local::now();
                let start = now.date_naive().and_hms_opt(0, 0, 0).unwrap();
                let end = now.date_naive().and_hms_opt(23, 59, 59).unwrap();
                
                if let Ok(stats) = self.app.query_handler()
                    .get_productivity_stats(
                        start.and_local_timezone(Local).unwrap(),
                        end.and_local_timezone(Local).unwrap()
                    ).await 
                {
                    state.update_productivity_stats(stats);
                }
            }
            AppEvent::ProjectCreated(project) | AppEvent::ProjectUpdated(project) => {
                if let Ok(projects) = self.app.query_handler().get_projects().await {
                    let mut state = self.state.write().await;
                    state.update_projects(projects);
                }
            }
            AppEvent::PomodoroStarted(session) => {
                let mut state = self.state.write().await;
                state.set_current_pomodoro(Some((*session).clone()));
            }
            AppEvent::PomodoroPaused(session) | AppEvent::PomodoroResumed(session) => {
                let mut state = self.state.write().await;
                state.set_current_pomodoro(Some((*session).clone()));
            }
            AppEvent::PomodoroCompleted(session) | AppEvent::PomodoroInterrupted(session) => {
                let mut state = self.state.write().await;
                state.set_current_pomodoro(None);
                
                // 更新番茄钟统计信息
                let now = Local::now();
                let start = now.date_naive().and_hms_opt(0, 0, 0).unwrap();
                let end = now.date_naive().and_hms_opt(23, 59, 59).unwrap();
                
                if let Ok(stats) = self.app.query_handler()
                    .get_pomodoro_stats(
                        start.and_local_timezone(Local).unwrap(),
                        end.and_local_timezone(Local).unwrap()
                    ).await 
                {
                    state.update_pomodoro_stats(stats);
                }
            }
            AppEvent::ConfigUpdated => {
                // 配置更新时可能需要刷新UI
            }
            _ => {}
        }
    }
} 