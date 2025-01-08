use iced::{
    widget::{Button, Column, Container},
    Element, Length,
};
use crate::presentation::ui::{Message, TimeTrackerApp, styles, Card};
use crate::core::models::Project;

pub fn view(app: &TimeTrackerApp) -> Element<Message> {
    let mut content = Column::new().spacing(20).padding(20);
    
    // 添加项目按钮
    let add_button = Button::new("添加项目")
        .style(styles::button::primary())
        .on_press(Message::OpenProjectDialog);
    content = content.push(add_button);
    
    // 项目列表
    for project in &app.projects {
        let project_card = Card::new()
            .title(&project.name)
            .subtitle(&format!("创建于: {}", project.created_at.format("%Y-%m-%d")))
            .content(project.description.as_deref().unwrap_or("无描述"))
            .on_press(Message::SelectProject(project.id));
        content = content.push(project_card);
    }
    
    Container::new(content)
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
}

// ... existing code ... 