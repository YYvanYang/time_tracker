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
}

pub struct ProjectView {
    app: Arc<App>,
    state: SharedState,
    name_input: String,
    description_input: String,
}

impl ProjectView {
    pub fn new(app: Arc<App>, state: SharedState) -> Self {
        Self {
            app,
            state,
            name_input: String::new(),
            description_input: String::new(),
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
        }
    }

    pub fn view(&self) -> Element<Message> {
        let input_row = Row::new()
            .push(
                TextInput::new("项目名称", &self.name_input)
                    .on_input(Message::NameChanged)
                    .padding(10),
            )
            .push(
                TextInput::new("描述", &self.description_input)
                    .on_input(Message::DescriptionChanged)
                    .padding(10),
            )
            .spacing(10);

        let control_row = Row::new()
            .push(
                Button::new(Text::new("创建项目"))
                    .on_press(Message::CreateProject)
                    .padding(10),
            )
            .spacing(10);

        let project_list = Scrollable::new(
            Column::new()
                .push(Text::new("项目列表").size(20))
                .push(Space::with_height(Length::Units(10)))
                .push(self.project_list())
                .spacing(10),
        );

        let content = Column::new()
            .push(input_row)
            .push(control_row)
            .push(project_list)
            .spacing(20)
            .padding(20);

        Container::new(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }

    fn project_list(&self) -> Element<Message> {
        let mut column = Column::new().spacing(10);

        // TODO: 从状态中获取项目列表并显示
        // for project in &self.state.projects {
        //     column = column.push(self.project_item(project));
        // }

        column.into()
    }

    fn project_item(&self, project: &Project) -> Element<Message> {
        Row::new()
            .push(Text::new(&project.name))
            .push(Space::with_width(Length::Fill))
            .push(
                Button::new(Text::new("选择"))
                    .on_press(Message::SelectProject(project.clone()))
                    .padding(5),
            )
            .push(
                Button::new(Text::new("编辑"))
                    .on_press(Message::UpdateProject(project.clone()))
                    .padding(5),
            )
            .push(
                Button::new(Text::new("删除"))
                    .on_press(Message::DeleteProject(project.clone()))
                    .padding(5),
            )
            .spacing(10)
            .into()
    }
} 