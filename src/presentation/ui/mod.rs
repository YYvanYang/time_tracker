use std::sync::Arc;
use iced::{
    widget::{Button, Column, Container, Row, Text},
    Element, Length, Sandbox, Settings, Theme,
};
use crate::core::{AppResult, models::*};
use crate::infrastructure::{
    config::Config,
    storage::Storage,
};
use crate::domain::{
    ActivityManager,
    ProjectManager,
    PomodoroManager,
    AnalysisManager,
    ExportManager,
};

mod components;
mod dialogs;
mod styles;
mod views;

pub use components::*;
pub use dialogs::*;
pub use styles::*;
pub use views::*;

pub struct TimeTrackerApp {
    config: Config,
    storage: Arc<dyn Storage + Send + Sync>,
    activity_manager: Arc<ActivityManager>,
    project_manager: Arc<ProjectManager>,
    pomodoro_manager: Arc<PomodoroManager>,
    analysis_manager: Arc<AnalysisManager>,
    export_manager: Arc<ExportManager>,
    current_view: View,
    dialog: Option<Box<dyn Dialog>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum View {
    Overview,
    Projects,
    Pomodoro,
    Statistics,
    Settings,
}

pub trait Dialog {
    fn view(&self) -> Element<Message>;
    fn update(&mut self, message: Message) -> DialogResult;
}

#[derive(Debug, Clone)]
pub enum Message {
    NoOp,
    ViewChanged(View),
    DialogClosed(DialogResult),
    // Add more messages as needed
}

pub enum DialogResult {
    None,
    Close,
    // Add more results as needed
}

impl TimeTrackerApp {
    pub fn new(
        config: Config,
        storage: Arc<dyn Storage + Send + Sync>,
        activity_manager: Arc<ActivityManager>,
        project_manager: Arc<ProjectManager>,
        pomodoro_manager: Arc<PomodoroManager>,
        analysis_manager: Arc<AnalysisManager>,
        export_manager: Arc<ExportManager>,
    ) -> Self {
        Self {
            config,
            storage,
            activity_manager,
            project_manager,
            pomodoro_manager,
            analysis_manager,
            export_manager,
            current_view: View::Overview,
            dialog: None,
        }
    }

    pub fn run() -> AppResult<()> {
        let settings = Settings::default();
        Ok(())
    }

    fn view(&self) -> Element<Message> {
        let content = match self.current_view {
            View::Overview => views::overview::view(),
            View::Projects => views::projects::view(),
            View::Pomodoro => views::pomodoro::view(),
            View::Statistics => views::statistics::view(),
            View::Settings => views::settings::view(),
        };

        let dialog = if let Some(dialog) = &self.dialog {
            dialog.view()
        } else {
            Container::new(Column::new()).into()
        };

        Container::new(
            Column::new()
                .push(self.view_navigation())
                .push(content)
                .push(dialog)
        )
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
    }

    fn view_navigation(&self) -> Element<Message> {
        let mut row = Row::new().spacing(20);

        for &view in &[View::Overview, View::Projects, View::Pomodoro, View::Statistics, View::Settings] {
            let label = match view {
                View::Overview => "Overview",
                View::Projects => "Projects",
                View::Pomodoro => "Pomodoro",
                View::Statistics => "Statistics",
                View::Settings => "Settings",
            };

            let button = Button::new(Text::new(label))
                .on_press(Message::ViewChanged(view))
                .style(if view == self.current_view {
                    styles::button::active()
                } else {
                    styles::button::primary()
                });

            row = row.push(button);
        }

        Container::new(row)
            .width(Length::Fill)
            .style(styles::container::header())
            .into()
    }
}