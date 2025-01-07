use crate::application::App;
use crate::core::models::{PomodoroSession, Project};
use crate::presentation::state::SharedState;
use iced::{
    widget::{Button, Column, Container, Row, Text, TextInput, PickList},
    Element, Length, Command, Subscription,
};
use std::sync::Arc;
use std::time::Duration;

#[derive(Debug, Clone)]
pub enum Message {
    DurationChanged(String),
    TagsChanged(String),
    ProjectSelected(Option<Project>),
    StartPomodoro,
    PausePomodoro,
    ResumePomodoro,
    StopPomodoro,
    Tick,
    ProjectsLoaded(Vec<Project>),
}

pub struct PomodoroView {
    app: Arc<App>,
    state: SharedState,
    duration_input: String,
    tags_input: String,
    selected_project: Option<Project>,
    available_projects: Vec<Project>,
}

impl PomodoroView {
    pub fn new(app: Arc<App>, state: SharedState) -> Self {
        Self {
            app,
            state,
            duration_input: String::from("25"),
            tags_input: String::new(),
            selected_project: None,
            available_projects: Vec::new(),
        }
    }

    pub fn update(&mut self, message: Message) -> Command<Message> {
        match message {
            Message::DurationChanged(duration) => {
                self.duration_input = duration;
                Command::none()
            }
            Message::TagsChanged(tags) => {
                self.tags_input = tags;
                Command::none()
            }
            Message::ProjectSelected(project) => {
                self.selected_project = project;
                Command::none()
            }
            Message::StartPomodoro => {
                let duration: u64 = self.duration_input.parse().unwrap_or(25);
                let tags: Vec<String> = self.tags_input
                    .split(',')
                    .map(|s| s.trim().to_string())
                    .filter(|s| !s.is_empty())
                    .collect();
                let project = self.selected_project.clone();
                let app = self.app.clone();

                Command::perform(
                    async move {
                        app.command_handler()
                            .start_pomodoro(project, Duration::from_secs(duration * 60), tags)
                            .await
                            .ok();
                    },
                    |_| Message::Tick,
                )
            }
            Message::PausePomodoro => {
                let state = self.state.clone();
                let app = self.app.clone();

                Command::perform(
                    async move {
                        let state = state.read().await;
                        if let Some(session) = &state.current_pomodoro {
                            app.command_handler().pause_pomodoro(session.clone()).await.ok();
                        }
                    },
                    |_| Message::Tick,
                )
            }
            Message::ResumePomodoro => {
                let state = self.state.clone();
                let app = self.app.clone();

                Command::perform(
                    async move {
                        let state = state.read().await;
                        if let Some(session) = &state.current_pomodoro {
                            app.command_handler().resume_pomodoro(session.clone()).await.ok();
                        }
                    },
                    |_| Message::Tick,
                )
            }
            Message::StopPomodoro => {
                let state = self.state.clone();
                let app = self.app.clone();

                Command::perform(
                    async move {
                        let state = state.read().await;
                        if let Some(session) = &state.current_pomodoro {
                            app.command_handler().interrupt_pomodoro(session.clone()).await.ok();
                        }
                    },
                    |_| Message::Tick,
                )
            }
            Message::Tick => Command::none(),
            Message::ProjectsLoaded(projects) => {
                self.available_projects = projects;
                Command::none()
            }
        }
    }

    pub fn view(&self) -> Element<Message> {
        let input_row = Row::new()
            .push(
                TextInput::new("时长(分钟)", &self.duration_input)
                    .on_input(Message::DurationChanged)
                    .padding(10),
            )
            .push(
                TextInput::new("标签(逗号分隔)", &self.tags_input)
                    .on_input(Message::TagsChanged)
                    .padding(10),
            )
            .push(
                PickList::new(
                    &self.available_projects,
                    self.selected_project.clone(),
                    Message::ProjectSelected,
                )
                .padding(10),
            )
            .spacing(10);

        let control_row = Row::new()
            .push(
                Button::new(Text::new("开始"))
                    .on_press(Message::StartPomodoro)
                    .padding(10),
            )
            .push(
                Button::new(Text::new("暂停"))
                    .on_press(Message::PausePomodoro)
                    .padding(10),
            )
            .push(
                Button::new(Text::new("继续"))
                    .on_press(Message::ResumePomodoro)
                    .padding(10),
            )
            .push(
                Button::new(Text::new("停止"))
                    .on_press(Message::StopPomodoro)
                    .padding(10),
            )
            .spacing(10);

        let timer_display = self.timer_display();

        let content = Column::new()
            .push(input_row)
            .push(control_row)
            .push(timer_display)
            .spacing(20)
            .padding(20);

        Container::new(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }

    fn timer_display(&self) -> Element<Message> {
        // TODO: 显示当前番茄钟状态和剩余时间
        Text::new("25:00").size(40).into()
    }

    pub fn subscription(&self) -> Subscription<Message> {
        iced::time::every(Duration::from_secs(1))
            .map(|_| Message::Tick)
    }
} 