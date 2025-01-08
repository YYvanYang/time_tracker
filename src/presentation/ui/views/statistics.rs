use iced::{
    widget::{Button, Column, Container, PickList, Row, Text},
    Element, Length,
};
use crate::presentation::ui::{Message, TimeTrackerApp, styles, Card};
use crate::core::models::{ProductivityStats, CategoryStats, PomodoroStats};

pub fn view(app: &TimeTrackerApp) -> Element<Message> {
    let mut content = Column::new().spacing(20).padding(20);
    
    // 生产力统计卡片
    let productivity_card = Card::new()
        .title("生产力统计")
        .content(format!(
            "总时长：{:02}:{:02}:{:02}\n专注时长：{:02}:{:02}:{:02}\n生产力得分：{:.1}%",
            app.productivity_stats.total_time / 3600,
            (app.productivity_stats.total_time % 3600) / 60,
            app.productivity_stats.total_time % 60,
            app.productivity_stats.productive_time / 3600,
            (app.productivity_stats.productive_time % 3600) / 60,
            app.productivity_stats.productive_time % 60,
            app.productivity_stats.productivity_score * 100.0
        ));
    content = content.push(productivity_card);
    
    // 类别统计卡片
    let category_card = Card::new()
        .title("类别统计")
        .content(app.category_stats.iter()
            .map(|stat| format!(
                "{}：{:02}:{:02}:{:02} ({:.1}%)",
                stat.category,
                stat.total_time / 3600,
                (stat.total_time % 3600) / 60,
                stat.total_time % 60,
                stat.percentage * 100.0
            ))
            .collect::<Vec<_>>()
            .join("\n"));
    content = content.push(category_card);
    
    // 番茄钟统计卡片
    let pomodoro_card = Card::new()
        .title("番茄钟统计")
        .content(format!(
            "总计：{}\n完成：{}\n总时长：{:02}:{:02}:{:02}\n平均时长：{:02}:{:02}:{:02}\n完成率：{:.1}%",
            app.pomodoro_stats.total_sessions,
            app.pomodoro_stats.completed_sessions,
            app.pomodoro_stats.total_time / 3600,
            (app.pomodoro_stats.total_time % 3600) / 60,
            app.pomodoro_stats.total_time % 60,
            app.pomodoro_stats.average_duration / 3600,
            (app.pomodoro_stats.average_duration % 3600) / 60,
            app.pomodoro_stats.average_duration % 60,
            app.pomodoro_stats.completion_rate * 100.0
        ));
    content = content.push(pomodoro_card);
    
    Container::new(content)
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
}