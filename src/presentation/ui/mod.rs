use std::sync::Arc;
use iced::{
    widget::{Button, Column, Container, Row, Text},
    Element, Length, Theme,
};
use crate::core::{AppResult, traits::Storage};
use crate::infrastructure::config::Config;

pub mod components;
pub mod dialogs;
pub mod styles;
pub mod views;

pub use components::*;
pub use dialogs::*;
pub use styles::*;
pub use views::*;

#[derive(Debug, Clone)]
pub enum Message {
    NoOp,
    Exit,
    ToggleWindow,
    ShowSettings,
    ShowAbout,
    ShowHelp,
    ShowExport,
    ShowImport,
    ShowBackup,
    ShowRestore,
    ShowPlugins,
    ShowLogs,
    ShowStats,
    ShowProjects,
    ShowTasks,
    ShowTimer,
    ShowCalendar,
    ShowReports,
    ShowNotifications,
    ShowUpdates,
    ShowFeedback,
    ShowBugs,
    ShowDonate,
    ShowLicense,
    ShowPrivacy,
    ShowTerms,
    ShowChangelog,
    ShowRoadmap,
    ShowContribute,
    ShowSponsors,
    ShowCommunity,
    ShowBlog,
    ShowDocs,
    ShowApi,
    ShowStatus,
    ShowMetrics,
    ShowHealth,
    ShowBackups,
    ShowLogs2,
    ShowStats2,
    ShowProjects2,
    ShowTasks2,
    ShowTimer2,
    ShowCalendar2,
    ShowReports2,
    ShowNotifications2,
    ShowUpdates2,
    ShowFeedback2,
    ShowBugs2,
    ShowDonate2,
    ShowLicense2,
    ShowPrivacy2,
    ShowTerms2,
    ShowChangelog2,
    ShowRoadmap2,
    ShowContribute2,
    ShowSponsors2,
    ShowCommunity2,
    ShowBlog2,
    ShowDocs2,
    ShowApi2,
    ShowStatus2,
    ShowMetrics2,
    ShowHealth2,
    ShowBackups2,
}

pub struct TimeTrackerApp {
    storage: Arc<dyn Storage + Send + Sync>,
    config: Config,
    state: State,
}

impl TimeTrackerApp {
    pub fn new(storage: Arc<dyn Storage + Send + Sync>, config: Config) -> Self {
        Self {
            storage,
            config,
            state: State::default(),
        }
    }

    pub fn view(&self) -> Element<Message> {
        let content = match self.state.current_view {
            View::Overview => self.overview_view(),
            View::Projects => self.projects_view(),
            View::Pomodoro => self.pomodoro_view(),
            View::Settings => self.settings_view(),
            View::Statistics => self.statistics_view(),
        };

        Container::new(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }

    fn overview_view(&self) -> Element<Message> {
        Column::new()
            .push(Text::new("概览").size(24))
            .spacing(20)
            .into()
    }

    fn projects_view(&self) -> Element<Message> {
        Column::new()
            .push(Text::new("项目").size(24))
            .spacing(20)
            .into()
    }

    fn pomodoro_view(&self) -> Element<Message> {
        Column::new()
            .push(Text::new("番茄钟").size(24))
            .spacing(20)
            .into()
    }

    fn settings_view(&self) -> Element<Message> {
        Column::new()
            .push(Text::new("设置").size(24))
            .spacing(20)
            .into()
    }

    fn statistics_view(&self) -> Element<Message> {
        Column::new()
            .push(Text::new("统计").size(24))
            .spacing(20)
            .into()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum View {
    Overview,
    Projects,
    Pomodoro,
    Settings,
    Statistics,
}

#[derive(Debug, Clone)]
pub struct State {
    current_view: View,
}

impl Default for State {
    fn default() -> Self {
        Self {
            current_view: View::Overview,
        }
    }
}