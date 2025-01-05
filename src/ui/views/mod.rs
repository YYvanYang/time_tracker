use crate::ui::components;

pub mod overview;
pub mod app_usage;
pub mod pomodoro;
pub mod projects;
pub mod statistics;
pub mod settings;

pub use overview::render as render_overview;
pub use app_usage::render as render_app_usage;
pub use pomodoro::render as render_pomodoro;
pub use projects::render as render_projects;
pub use statistics::render as render_statistics;
pub use settings::render as render_settings; 