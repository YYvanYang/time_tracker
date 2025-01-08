use crate::application::App;
use crate::core::models::Project;
use crate::presentation::state::SharedState;
use iced::{
    widget::{Button, Column, Container, Row, Text, TextInput, Scrollable, Space},
    Element, Length, Command,
};
use std::sync::Arc;

#[derive(Debug, Clone)]
pub enum Message {
    NameChanged(String),
    DescriptionChanged(String),
    CreateProject,
    UpdateProject(Project),
    DeleteProject(Project),
    SelectProject(Project),
    ProjectsLoaded(Vec<Project>),
    EditProject(Project),
}

pub struct ProjectView {
    app: Arc<App>,
    state: SharedState,
    name_input: String,
    description_input: String,
    projects: Vec<Project>,
}

impl ProjectView {
    pub fn new(app: Arc<App>, state: SharedState) -> Self {
        Self {
            app,
            state,
            name_input: String::new(),
            description_input: String::new(),
            projects: Vec::new(),
        }
    }

    pub fn update(&mut self, message: Message) -> Command<Message> {
        match message {
            Message::NameChanged(name) => {
                self.name_input = name;
                Command::none()
            }
            Message::DescriptionChanged(description) => {
                self.description_input = description;
                Command::none()
            }
            Message::CreateProject => {
                let name = self.name_input.clone();
                let description = Some(self.description_input.clone());
                let app = self.app.clone();
                
                Command::perform(
                    async move {
                        app.command_handler().create_project(name, description).await.ok();
                        app.query_handler().get_projects().await.unwrap_or_default()
                    },
                    Message::ProjectsLoaded,
                )
            }
            Message::UpdateProject(project) => {
                let app = self.app.clone();
                
                Command::perform(
                    async move {
                        app.command_handler().update_project(project).await.ok();
                        app.query_handler().get_projects().await.unwrap_or_default()
                    },
                    Message::ProjectsLoaded,
                )
            }
            Message::DeleteProject(project) => {
                let app = self.app.clone();
                
                Command::perform(
                    async move {
                        app.command_handler().delete_project(project).await.ok();
                        app.query_handler().get_projects().await.unwrap_or_default()
                    },
                    Message::ProjectsLoaded,
                )
            }
            Message::SelectProject(project) => {
                let state = self.state.clone();
                
                Command::perform(
                    async move {
                        let mut state = state.write().await;
                        state.set_selected_project(Some(project));
                    },
                    |_| Message::ProjectsLoaded(vec![]),
                )
            }
            Message::ProjectsLoaded(projects) => {
                let state = self.state.clone();
                
                Command::perform(
                    async move {
                        let mut state = state.write().await;
                        state.update_projects(projects);
                    },
                    |_| Message::ProjectsLoaded(vec![]),
                )
            }
            Message::EditProject(project) => {
                Command::none()
            }
        }
    }

    pub fn view(&self) -> Element<Message> {
        let content = Column::new()
            .push(Text::new("项目管理").size(24))
            .push(Space::with_height(Length::Fixed(20.0)))
            .push(
                Row::new()
                    .push(
                        TextInput::new("项目名称", &self.name_input)
                            .on_input(Message::NameChanged)
                            .width(Length::Fixed(200.0))
                    )
                    .push(Space::with_width(Length::Fixed(10.0)))
                    .push(
                        TextInput::new("项目描述", &self.description_input)
                            .on_input(Message::DescriptionChanged)
                            .width(Length::Fixed(300.0))
                    )
                    .push(Space::with_width(Length::Fixed(10.0)))
                    .push(
                        Button::new(Text::new("创建"))
                            .on_press(Message::CreateProject)
                            .style(iced::theme::Button::Primary)
                    )
            )
            .push(Space::with_height(Length::Fixed(20.0)))
            .push(self.project_list())
            .spacing(10);

        Container::new(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .padding(20)
            .into()
    }

    fn project_list(&self) -> Element<Message> {
        let mut column = Column::new().spacing(10);

        for project in &self.projects {
            column = column.push(self.project_item(project));
        }

        column = column.push(Space::with_height(Length::Fixed(10)));

        column.into()
    }

    fn project_item<'a>(&'a self, project: &'a Project) -> Element<'a, Message> {
        Row::new()
            .push(Text::new(&project.name))
            .push(Space::with_width(Length::Fill))
            .push(
                Button::new(Text::new("编辑"))
                    .on_press(Message::EditProject(project.clone()))
                    .style(iced::theme::Button::Secondary)
            )
            .push(
                Button::new(Text::new("删除"))
                    .on_press(Message::DeleteProject(project.clone()))
                    .style(iced::theme::Button::Destructive)
            )
            .spacing(10)
            .into()
    }
} 