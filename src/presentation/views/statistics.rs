use crate::application::App;
use crate::core::models::Project;
use crate::domain::analysis::{ProductivityStats, CategoryStats, PomodoroStats};
use crate::presentation::state::SharedState;
use iced::{
    widget::{Button, Column, Container, Row, Text, PickList, Space},
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
    Custom,
}

impl std::fmt::Display for TimeRange {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TimeRange::Today => write!(f, "今天"),
            TimeRange::Week => write!(f, "本周"),
            TimeRange::Month => write!(f, "本月"),
            TimeRange::Year => write!(f, "今年"),
            TimeRange::Custom => write!(f, "自定义"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExportFormat {
    CSV,
    JSON,
}

impl std::fmt::Display for ExportFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ExportFormat::CSV => write!(f, "CSV"),
            ExportFormat::JSON => write!(f, "JSON"),
        }
    }
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
            selected_range: TimeRange::Today,
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
            Message::StatsLoaded { .. } => Command::none(),
            Message::ExportData(format) => {
                let (start, end) = self.get_time_range();
                match format {
                    ExportFormat::CSV => {
                        // TODO: 导出CSV
                    }
                    ExportFormat::JSON => {
                        // TODO: 导出JSON
                    }
                }
                Command::none()
            }
        }
    }

    fn load_stats(&self) -> Command<Message> {
        let app = Arc::clone(&self.app);
        let (start, end) = self.get_time_range();
        Command::perform(async move {
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
        }, |msg| msg)
    }

    fn get_time_range(&self) -> (chrono::DateTime<Local>, chrono::DateTime<Local>) {
        let now = Local::now();
        let start = match self.selected_range {
            TimeRange::Today => now.date_naive().and_hms_opt(0, 0, 0).unwrap(),
            TimeRange::Week => (now - Duration::days(7)).date_naive().and_hms_opt(0, 0, 0).unwrap(),
            TimeRange::Month => (now - Duration::days(30)).date_naive().and_hms_opt(0, 0, 0).unwrap(),
            TimeRange::Year => (now - Duration::days(365)).date_naive().and_hms_opt(0, 0, 0).unwrap(),
            TimeRange::Custom => now.date_naive().and_hms_opt(0, 0, 0).unwrap(), // TODO: 实现自定义日期选择
        };
        let end = now;
        (start.and_local_timezone(Local).unwrap(), end)
    }

    pub fn view(&self) -> Element<Message> {
        let content = Column::new()
            .push(Text::new("统计分析").size(24))
            .push(Space::with_height(Length::Fixed(20.0)))
            .push(
                Row::new()
                    .push(
                        PickList::new(
                            &self.available_projects,
                            self.selected_project.clone(),
                            |project| Message::ProjectSelected(Some(project)),
                        )
                        .placeholder("选择项目")
                        .width(Length::Fixed(200.0))
                    )
                    .push(Space::with_width(Length::Fixed(10.0)))
                    .push(
                        PickList::new(
                            &[
                                TimeRange::Today,
                                TimeRange::Week,
                                TimeRange::Month,
                                TimeRange::Year,
                                TimeRange::Custom,
                            ],
                            Some(self.selected_range),
                            Message::TimeRangeSelected,
                        )
                        .width(Length::Fixed(120.0))
                    )
            )
            .push(Space::with_height(Length::Fixed(20.0)))
            .push(self.stats_display())
            .spacing(10);

        Container::new(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .padding(20)
            .into()
    }

    fn stats_display(&self) -> Element<Message> {
        Column::new()
            .push(Text::new("统计数据").size(20))
            .push(Space::with_height(Length::Fixed(10.0)))
            .into()
    }
} 