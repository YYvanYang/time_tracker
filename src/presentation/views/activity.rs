use crate::application::App;
use crate::core::models::Activity;
use crate::presentation::state::SharedState;
use iced::{
    widget::{Button, Column, Container, Row, Text, TextInput},
    Element, Length, Subscription, Command,
};
use std::sync::Arc;
use std::time::Duration;

#[derive(Debug, Clone)]
pub enum Message {
    NameChanged(String),
    CategoryChanged(String),
    StartActivity,
    StopActivity,
    MarkAsProductive,
    MarkAsUnproductive,
    Tick,
}

pub struct ActivityView {
    app: Arc<App>,
    state: SharedState,
    name_input: String,
    category_input: String,
}

impl ActivityView {
    pub fn new(app: Arc<App>, state: SharedState) -> Self {
        Self {
            app,
            state,
            name_input: String::new(),
            category_input: String::new(),
        }
    }

    pub fn update(&mut self, message: Message) -> Command<Message> {
        match message {
            Message::NameChanged(name) => {
                self.name_input = name;
                Command::none()
            }
            Message::CategoryChanged(category) => {
                self.category_input = category;
                Command::none()
            }
            Message::StartActivity => {
                let name = self.name_input.clone();
                let category = self.category_input.clone();
                
                Command::perform(
                    self.app.command_handler().start_activity(name, category),
                    |_| Message::Tick,
                )
            }
            Message::StopActivity => {
                let state = self.state.clone();
                
                Command::perform(
                    async move {
                        let state = state.read().await;
                        if let Some(activity) = &state.current_activity {
                            // TODO: 停止活动
                        }
                    },
                    |_| Message::Tick,
                )
            }
            Message::MarkAsProductive => {
                let state = self.state.clone();
                
                Command::perform(
                    async move {
                        let state = state.read().await;
                        if let Some(activity) = &state.current_activity {
                            // TODO: 标记为生产性活动
                        }
                    },
                    |_| Message::Tick,
                )
            }
            Message::MarkAsUnproductive => {
                let state = self.state.clone();
                
                Command::perform(
                    async move {
                        let state = state.read().await;
                        if let Some(activity) = &state.current_activity {
                            // TODO: 标记为非生产性活动
                        }
                    },
                    |_| Message::Tick,
                )
            }
            Message::Tick => Command::none(),
        }
    }

    pub fn view(&self) -> Element<Message> {
        let input_row = Row::new()
            .push(
                TextInput::new("活动名称", &self.name_input)
                    .on_input(Message::NameChanged)
                    .padding(10),
            )
            .push(
                TextInput::new("类别", &self.category_input)
                    .on_input(Message::CategoryChanged)
                    .padding(10),
            )
            .spacing(10);

        let control_row = Row::new()
            .push(
                Button::new(Text::new("开始活动"))
                    .on_press(Message::StartActivity)
                    .padding(10),
            )
            .push(
                Button::new(Text::new("停止活动"))
                    .on_press(Message::StopActivity)
                    .padding(10),
            )
            .spacing(10);

        let productivity_row = Row::new()
            .push(
                Button::new(Text::new("标记为生产性"))
                    .on_press(Message::MarkAsProductive)
                    .padding(10),
            )
            .push(
                Button::new(Text::new("标记为非生产性"))
                    .on_press(Message::MarkAsUnproductive)
                    .padding(10),
            )
            .spacing(10);

        let content = Column::new()
            .push(input_row)
            .push(control_row)
            .push(productivity_row)
            .spacing(20)
            .padding(20);

        Container::new(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }

    pub fn subscription(&self) -> Subscription<Message> {
        iced::time::every(Duration::from_secs(1))
            .map(|_| Message::Tick)
    }
} 