use crate::application::App;
use crate::core::models::Project;
use crate::domain::analysis::{ProductivityStats, CategoryStats, PomodoroStats};
use crate::presentation::state::SharedState;
use iced::{
    widget::{Button, Column, Container, Row, Text, PickList, Space, Canvas},
    Element, Length, Command,
};
use std::sync::Arc;
use chrono::{Local, Duration};

#[derive(Debug, Clone)]
pub enum Message {
    ProjectSelected(Option<Project>),
    TimeRangeSelected(TimeRange),
    StatsLoaded {
        productivity: ProductivityStats,
        categories: Vec<CategoryStats>,
        pomodoro: PomodoroStats,
    },
    ExportData(ExportFormat),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TimeRange {
    Today,
    Week,
    Month,
    Year,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExportFormat {
    CSV,
    JSON,
}

pub struct StatisticsView {
    app: Arc<App>,
    state: SharedState,
    selected_project: Option<Project>,
    selected_range: TimeRange,
    available_projects: Vec<Project>,
}

impl StatisticsView {
    pub fn new(app: Arc<App>, state: SharedState) -> Self {
        Self {
            app,
            state,
            selected_project: None,
            selected_range: TimeRange::Week,
            available_projects: Vec::new(),
        }
    }

    pub fn update(&mut self, message: Message) -> Command<Message> {
        match message {
            Message::ProjectSelected(project) => {
                self.selected_project = project;
                self.load_stats()
            }
            Message::TimeRangeSelected(range) => {
                self.selected_range = range;
                self.load_stats()
            }
            Message::StatsLoaded { .. } => {
                // 更新状态
                Command::none()
            }
            Message::ExportData(format) => {
                let app = self.app.clone();
                let (start, end) = self.get_time_range();
                
                Command::perform(
                    async move {
                        match format {
                            ExportFormat::CSV => {
                                app.query_handler().export_activities_csv(start, end).await.ok();
                            }
                            ExportFormat::JSON => {
                                app.query_handler().export_json(start, end).await.ok();
                            }
                        }
                    },
                    |_| Message::TimeRangeSelected(TimeRange::Week),
                )
            }
        }
    }

    fn load_stats(&self) -> Command<Message> {
        let app = self.app.clone();
        let (start, end) = self.get_time_range();
        
        Command::perform(
            async move {
                let productivity = app.query_handler()
                    .get_productivity_stats(start, end)
                    .await
                    .unwrap_or_default();
                    
                let categories = app.query_handler()
                    .get_category_stats(start, end)
                    .await
                    .unwrap_or_default();
                    
                let pomodoro = app.query_handler()
                    .get_pomodoro_stats(start, end)
                    .await
                    .unwrap_or_default();
                    
                Message::StatsLoaded {
                    productivity,
                    categories,
                    pomodoro,
                }
            },
            |msg| msg,
        )
    }

    fn get_time_range(&self) -> (chrono::DateTime<Local>, chrono::DateTime<Local>) {
        let now = Local::now();
        let end = now;
        
        let start = match self.selected_range {
            TimeRange::Today => now.date_naive().and_hms_opt(0, 0, 0).unwrap(),
            TimeRange::Week => (now - Duration::days(7)).date_naive().and_hms_opt(0, 0, 0).unwrap(),
            TimeRange::Month => (now - Duration::days(30)).date_naive().and_hms_opt(0, 0, 0).unwrap(),
            TimeRange::Year => (now - Duration::days(365)).date_naive().and_hms_opt(0, 0, 0).unwrap(),
        };
        
        (start.and_local_timezone(Local).unwrap(), end)
    }

    pub fn view(&self) -> Element<Message> {
        let controls = Row::new()
            .push(
                PickList::new(
                    &self.available_projects,
                    self.selected_project.clone(),
                    Message::ProjectSelected,
                )
                .padding(10),
            )
            .push(
                PickList::new(
                    &[
                        TimeRange::Today,
                        TimeRange::Week,
                        TimeRange::Month,
                        TimeRange::Year,
                    ],
                    Some(self.selected_range),
                    Message::TimeRangeSelected,
                )
                .padding(10),
            )
            .spacing(10);

        let export_buttons = Row::new()
            .push(
                Button::new(Text::new("导出 CSV"))
                    .on_press(Message::ExportData(ExportFormat::CSV))
                    .padding(10),
            )
            .push(
                Button::new(Text::new("导出 JSON"))
                    .on_press(Message::ExportData(ExportFormat::JSON))
                    .padding(10),
            )
            .spacing(10);

        let stats_display = self.stats_display();

        let content = Column::new()
            .push(controls)
            .push(stats_display)
            .push(export_buttons)
            .spacing(20)
            .padding(20);

        Container::new(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }

    fn stats_display(&self) -> Element<Message> {
        // TODO: 显示统计信息和图表
        Column::new()
            .push(Text::new("统计信息").size(20))
            .push(Space::with_height(Length::Units(20)))
            .into()
    }
} 