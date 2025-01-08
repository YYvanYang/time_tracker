pub mod core;
pub mod domain;
pub mod application;
pub mod infrastructure;
pub mod presentation;
pub mod plugins;

pub use core::error::{AppError, AppResult};
pub use core::models::*;
pub use core::traits::*;

// Re-export domain managers
pub use domain::{
    ActivityManager,
    ProjectManager,
    PomodoroManager,
    AnalysisManager,
    ExportManager,
}; 