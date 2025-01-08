//src/ui/views/pomodoro.rs

use iced::{
    widget::{Button, Column, Container, ProgressBar, Row, Text},
    Element, Length,
};
use crate::presentation::ui::{Message, TimeTrackerApp, styles, Card};
use crate::core::models::{PomodoroSession, PomodoroStatus};

pub fn view(app: &TimeTrackerApp) -> Element<Message> {
    let mut content = Column::new().spacing(20).padding(20);
    
    // 番茄钟状态卡片
    let status_card = Card::new()
        .title("当前状态")
        .content(match app.pomodoro_state {
            PomodoroStatus::Working => "专注工作中",
            PomodoroStatus::ShortBreak => "短休息中",
            PomodoroStatus::LongBreak => "长休息中",
            PomodoroStatus::Paused(_) => "已暂停",
            PomodoroStatus::Idle => "未开始",
            PomodoroStatus::Completed => "已完成",
        });
    content = content.push(status_card);
    
    // 进度条
    if let Some(progress) = app.pomodoro_progress {
        content = content.push(
            Container::new(
                ProgressBar::new(0.0..=1.0, progress)
                    .width(Length::Fill)
            )
            .width(Length::Fill)
            .padding(10)
        );
    }
    
    // 控制按钮
    let mut controls = Row::new().spacing(10);
    
    match app.pomodoro_state {
        PomodoroStatus::Idle => {
            let start_button = Button::new(Text::new("开始专注"))
                .style(styles::button::primary())
                .on_press(Message::StartPomodoro);
            controls = controls.push(start_button);
        }
        PomodoroStatus::Working |
        PomodoroStatus::ShortBreak |
        PomodoroStatus::LongBreak => {
            let pause_button = Button::new(Text::new("暂停"))
                .style(styles::button::primary())
                .on_press(Message::PausePomodoro);
            controls = controls.push(pause_button);
            
            let stop_button = Button::new(Text::new("停止"))
                .style(styles::button::primary())
                .on_press(Message::StopPomodoro);
            controls = controls.push(stop_button);
        }
        PomodoroStatus::Paused(_) => {
            let resume_button = Button::new(Text::new("继续"))
                .style(styles::button::primary())
                .on_press(Message::ResumePomodoro);
            controls = controls.push(resume_button);
            
            let stop_button = Button::new(Text::new("停止"))
                .style(styles::button::primary())
                .on_press(Message::StopPomodoro);
            controls = controls.push(stop_button);
        }
        _ => {}
    }
    
    content = content.push(
        Container::new(controls)
            .width(Length::Fill)
            .center_x()
            .padding(10)
    );
    
    // 统计卡片
    let stats_card = Card::new()
        .title("今日统计")
        .content(format!(
            "完成番茄钟：{}\n专注时长：{:02}:{:02}:{:02}",
            app.pomodoro_stats.completed_sessions,
            app.pomodoro_stats.total_time / 3600,
            (app.pomodoro_stats.total_time % 3600) / 60,
            app.pomodoro_stats.total_time % 60
        ));
    content = content.push(stats_card);
    
    Container::new(content)
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
}

                                        