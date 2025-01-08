use iced::{
    widget::{Button, Column, Container, Row, Text, TextInput},
    Element, Length,
};
use crate::core::models::Project;
use crate::presentation::ui::{Message, styles};
use super::base::{Dialog, DialogContainer};

pub struct ProjectDialog {
    project: Project,
    name_input: String,
    description_input: String,
}

impl ProjectDialog {
    pub fn new() -> Self {
        Self {
            project: Project::new(String::new(), None),
            name_input: String::new(),
            description_input: String::new(),
        }
    }

    pub fn edit(project: Project) -> Self {
        Self {
            name_input: project.name.clone(),
            description_input: project.description.clone().unwrap_or_default(),
            project,
        }
    }
}

impl Dialog for ProjectDialog {
    fn view(&self) -> Element<Message> {
        let content = Column::new()
            .spacing(20)
            .push(Text::new("Project").size(24))
            .push(
                Column::new()
                    .spacing(10)
                    .push(Text::new("Name"))
                    .push(
                        TextInput::new("Enter project name", &self.name_input)
                            .padding(10)
                            .width(Length::Fill),
                    ),
            )
            .push(
                Column::new()
                    .spacing(10)
                    .push(Text::new("Description"))
                    .push(
                        TextInput::new("Enter project description", &self.description_input)
                            .padding(10)
                            .width(Length::Fill),
                    ),
            )
            .push(
                Row::new()
                    .spacing(10)
                    .push(
                        Button::new(Text::new("Cancel"))
                            .style(styles::button::primary()),
                    )
                    .push(
                        Button::new(Text::new("Save"))
                            .style(styles::button::primary()),
                    ),
            );

        DialogContainer::new()
            .push(content)
            .spacing(20)
            .into_element()
    }

    fn update(&mut self, message: Message) {
        // TODO: 实现更新逻辑
    }
} 