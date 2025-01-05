use crate::error::Result;
use std::time::Duration;

pub struct ShutdownManager {
    scheduled_time: Option<chrono::DateTime<chrono::Local>>,
    pomodoro_count: u32,
    target_count: u32,
}

impl ShutdownManager {
    pub fn new(target_count: u32) -> Self {
        Self {
            scheduled_time: None,
            pomodoro_count: 0,
            target_count,
        }
    }

    pub fn increment_pomodoro_count(&mut self) {
        self.pomodoro_count += 1;
    }

    pub fn should_shutdown(&self) -> bool {
        self.pomodoro_count >= self.target_count
    }

    pub fn schedule_shutdown(&mut self, delay: Duration) -> Result<()> {
        self.scheduled_time = Some(chrono::Local::now() + chrono::Duration::from_std(delay)?);
        Ok(())
    }
} 