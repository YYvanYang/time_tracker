use crate::error::{Result, TimeTrackerError};
use chrono::{DateTime, Local};
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use crate::config;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum PreviousState {
    Working,
    ShortBreak,
    LongBreak,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum PomodoroState {
    Idle,
    Working,
    ShortBreak,
    LongBreak,
    Paused(PreviousState),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PomodoroConfig {
    pub work_duration: Duration,
    pub short_break_duration: Duration,
    pub long_break_duration: Duration,
    pub long_break_interval: u32,
    pub auto_start_breaks: bool,
    pub auto_start_pomodoros: bool,
    pub sound_enabled: bool,
    pub sound_volume: u8,
}

impl Default for PomodoroConfig {
    fn default() -> Self {
        Self {
            work_duration: Duration::from_secs(25 * 60),
            short_break_duration: Duration::from_secs(5 * 60),
            long_break_duration: Duration::from_secs(15 * 60),
            long_break_interval: 4,
            auto_start_breaks: false,
            auto_start_pomodoros: false,
            sound_enabled: true,
            sound_volume: 80,
        }
    }
}

impl From<config::PomodoroConfig> for PomodoroConfig {
    fn from(config: config::PomodoroConfig) -> Self {
        Self {
            work_duration: config.work_duration,
            short_break_duration: config.short_break_duration,
            long_break_duration: config.long_break_duration,
            long_break_interval: config.long_break_interval,
            auto_start_breaks: config.auto_start_breaks,
            auto_start_pomodoros: config.auto_start_pomodoros,
            sound_enabled: true,
            sound_volume: 80,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PomodoroStats {
    pub total_completed: u32,
    pub total_interrupted: u32,
    pub total_work_time: Duration,
    pub total_break_time: Duration,
    pub longest_streak: u32,
    pub current_streak: u32,
    pub daily_completed: Vec<(DateTime<Local>, u32)>,
}

impl Default for PomodoroStats {
    fn default() -> Self {
        Self {
            total_completed: 0,
            total_interrupted: 0,
            total_work_time: Duration::from_secs(0),
            total_break_time: Duration::from_secs(0),
            longest_streak: 0,
            current_streak: 0,
            daily_completed: Vec::new(),
        }
    }
}

#[derive(Debug)]
pub struct PomodoroTimer {
    config: PomodoroConfig,
    state: PomodoroState,
    start_time: Option<Instant>,
    remaining_time: Duration,
    completed_count: u32,
    stats: Arc<Mutex<PomodoroStats>>,
    callbacks: PomodoroCallbacks,
}

pub struct PomodoroCallbacks {
    pub on_tick: Arc<dyn Fn(Duration) + Send + Sync>,
    pub on_complete: Arc<dyn Fn() + Send + Sync>,
    pub on_pause: Arc<dyn Fn() + Send + Sync>,
    pub on_resume: Arc<dyn Fn() + Send + Sync>,
}

impl Default for PomodoroCallbacks {
    fn default() -> Self {
        Self {
            on_tick: Arc::new(|_| {}),
            on_complete: Arc::new(|| {}),
            on_pause: Arc::new(|| {}),
            on_resume: Arc::new(|| {}),
        }
    }
}

impl std::fmt::Debug for PomodoroCallbacks {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PomodoroCallbacks")
            .field("on_tick", &"<callback>")
            .field("on_complete", &"<callback>")
            .field("on_pause", &"<callback>")
            .field("on_resume", &"<callback>")
            .finish()
    }
}

impl PomodoroTimer {
    pub fn new(config: PomodoroConfig, callbacks: PomodoroCallbacks) -> Self {
        Self {
            config: config.clone(),
            callbacks,
            state: PomodoroState::Idle,
            start_time: None,
            remaining_time: config.work_duration,
            completed_count: 0,
            stats: Arc::new(Mutex::new(PomodoroStats::default())),
        }
    }

    pub fn start(&mut self) -> Result<()> {
        match self.state {
            PomodoroState::Idle | PomodoroState::Paused(_) => {
                self.state = PomodoroState::Working;
                self.remaining_time = self.config.work_duration;
                self.start_time = Some(Instant::now());
                Ok(())
            }
            _ => Err(TimeTrackerError::Platform("Timer is already running".into())),
        }
    }

    pub fn pause(&mut self) -> Result<()> {
        match self.state {
            PomodoroState::Working => {
                self.state = PomodoroState::Paused(PreviousState::Working);
                self.update_remaining_time()?;
                self.start_time = None;
            }
            PomodoroState::ShortBreak => {
                self.state = PomodoroState::Paused(PreviousState::ShortBreak);
                self.update_remaining_time()?;
                self.start_time = None;
            }
            PomodoroState::LongBreak => {
                self.state = PomodoroState::Paused(PreviousState::LongBreak);
                self.update_remaining_time()?;
                self.start_time = None;
            }
            _ => {
                return Err(TimeTrackerError::Platform(
                    "Timer is not running".into()
                ));
            }
        }
        Ok(())
    }

    pub fn stop(&mut self) -> Result<()> {
        match self.state {
            PomodoroState::Working => {
                self.handle_interruption()?;
            }
            _ => {
                self.state = PomodoroState::Idle;
                self.remaining_time = self.config.work_duration;
                self.start_time = None;
            }
        }
        Ok(())
    }

    pub fn update(&mut self) -> Result<()> {
        match self.state {
            PomodoroState::Working | PomodoroState::ShortBreak | PomodoroState::LongBreak => {
                if let Some(start) = self.start_time {
                    let elapsed = start.elapsed();
                    if elapsed >= self.remaining_time {
                        self.handle_completion()?;
                    } else {
                        self.remaining_time = self.remaining_time.saturating_sub(elapsed);
                        (self.callbacks.on_tick)(self.remaining_time);
                    }
                }
            }
            _ => {}
        }
        Ok(())
    }

    fn handle_completion(&mut self) -> Result<()> {
        match self.state {
            PomodoroState::Working => {
                self.completed_count += 1;
                
                // 更新统计信息
                let mut stats = self.stats.lock().map_err(|_| {
                    TimeTrackerError::Platform("Failed to lock stats".into())
                })?;
                
                stats.total_completed += 1;
                stats.total_work_time += self.config.work_duration;
                stats.current_streak += 1;
                stats.longest_streak = stats.longest_streak.max(stats.current_streak);
                
                // 更新每日完成数
                let today = Local::now();
                if let Some(last) = stats.daily_completed.last_mut() {
                    if last.0.date_naive() == today.date_naive() {
                        last.1 += 1;
                    } else {
                        stats.daily_completed.push((today, 1));
                    }
                } else {
                    stats.daily_completed.push((today, 1));
                }

                // 决定下一个状态
                if self.completed_count >= self.config.long_break_interval {
                    self.state = if self.config.auto_start_breaks {
                        PomodoroState::LongBreak
                    } else {
                        PomodoroState::Idle
                    };
                    self.completed_count = 0;
                    self.remaining_time = self.config.long_break_duration;
                } else {
                    self.state = if self.config.auto_start_breaks {
                        PomodoroState::ShortBreak
                    } else {
                        PomodoroState::Idle
                    };
                    self.remaining_time = self.config.short_break_duration;
                }

                (self.callbacks.on_complete)();
            }
            PomodoroState::ShortBreak | PomodoroState::LongBreak => {
                let mut stats = self.stats.lock().map_err(|_| {
                    TimeTrackerError::Platform("Failed to lock stats".into())
                })?;
                
                stats.total_break_time += match self.state {
                    PomodoroState::ShortBreak => self.config.short_break_duration,
                    PomodoroState::LongBreak => self.config.long_break_duration,
                    _ => unreachable!(),
                };

                self.state = if self.config.auto_start_pomodoros {
                    PomodoroState::Working
                } else {
                    PomodoroState::Idle
                };
                self.remaining_time = self.config.work_duration;

                (self.callbacks.on_complete)();
            }
            _ => {}
        }

        self.start_time = if self.state != PomodoroState::Idle {
            Some(Instant::now())
        } else {
            None
        };

        Ok(())
    }

    fn handle_interruption(&mut self) -> Result<()> {
        let mut stats = self.stats.lock().map_err(|_| {
            TimeTrackerError::Platform("Failed to lock stats".into())
        })?;

        stats.total_interrupted += 1;
        stats.current_streak = 0;

        self.state = PomodoroState::Idle;
        self.remaining_time = self.config.work_duration;
        self.start_time = None;

        Ok(())
    }

    fn update_remaining_time(&mut self) -> Result<()> {
        if let Some(start) = self.start_time {
            self.remaining_time = self.remaining_time.saturating_sub(start.elapsed());
        }
        Ok(())
    }

    pub fn get_state(&self) -> PomodoroState {
        self.state.clone()
    }

    pub fn get_remaining_time(&self) -> Duration {
        self.remaining_time
    }

    pub fn get_progress(&self) -> f32 {
        let total_duration = match self.state {
            PomodoroState::Working => self.config.work_duration,
            PomodoroState::ShortBreak => self.config.short_break_duration,
            PomodoroState::LongBreak => self.config.long_break_duration,
            PomodoroState::Paused(ref prev_state) => match prev_state {
                PreviousState::Working => self.config.work_duration,
                PreviousState::ShortBreak => self.config.short_break_duration,
                PreviousState::LongBreak => self.config.long_break_duration,
            },
            PomodoroState::Idle => return 0.0,
        };

        1.0 - (self.remaining_time.as_secs_f32() / total_duration.as_secs_f32())
    }

    pub fn get_stats(&self) -> Result<PomodoroStats> {
        Ok(self.stats.lock().map_err(|_| {
            TimeTrackerError::Platform("Failed to lock stats".into())
        })?.clone())
    }

    fn validate_state_transition(&self, new_state: &PomodoroState) -> Result<()> {
        match (&self.state, new_state) {
            (PomodoroState::Paused(prev), PomodoroState::Working) if matches!(prev, PreviousState::Working) => Ok(()),
            (PomodoroState::Paused(prev), PomodoroState::ShortBreak) if matches!(prev, PreviousState::ShortBreak) => Ok(()),
            (PomodoroState::Paused(prev), PomodoroState::LongBreak) if matches!(prev, PreviousState::LongBreak) => Ok(()),
            (PomodoroState::Idle, PomodoroState::Working) => Ok(()),
            (PomodoroState::Working, PomodoroState::ShortBreak) => Ok(()),
            (PomodoroState::Working, PomodoroState::LongBreak) => Ok(()),
            (PomodoroState::ShortBreak, PomodoroState::Working) => Ok(()),
            (PomodoroState::LongBreak, PomodoroState::Working) => Ok(()),
            _ => Err(TimeTrackerError::State(
                format!("Invalid state transition: {:?} -> {:?}", self.state, new_state)
            )),
        }
    }

    pub fn update_config(&mut self, config: PomodoroConfig) {
        self.config = config;
        // 如果当前正在工作，更新剩余时间
        match self.state {
            PomodoroState::Working => {
                self.remaining_time = self.config.work_duration;
            }
            PomodoroState::ShortBreak => {
                self.remaining_time = self.config.short_break_duration;
            }
            PomodoroState::LongBreak => {
                self.remaining_time = self.config.long_break_duration;
            }
            PomodoroState::Paused(ref prev_state) => {
                match prev_state {
                    PreviousState::Working => {
                        self.remaining_time = self.config.work_duration;
                    }
                    PreviousState::ShortBreak => {
                        self.remaining_time = self.config.short_break_duration;
                    }
                    PreviousState::LongBreak => {
                        self.remaining_time = self.config.long_break_duration;
                    }
                }
            }
            PomodoroState::Idle => {
                self.remaining_time = self.config.work_duration;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU32, Ordering};
    use std::time::Duration;

    fn create_test_timer() -> PomodoroTimer {
        let config = PomodoroConfig {
            work_duration: Duration::from_secs(2),
            short_break_duration: Duration::from_secs(1),
            long_break_duration: Duration::from_secs(3),
            long_break_interval: 2,
            ..PomodoroConfig::default()
        };

        let completed_count = Arc::new(AtomicU32::new(0));
        let completed_count_clone = completed_count.clone();

        let callbacks = PomodoroCallbacks {
            on_complete: Arc::new(move || {
                completed_count.fetch_add(1, Ordering::SeqCst);
            }),
            on_tick: Arc::new(|_| {}),
            on_pause: Arc::new(|| {}),
            on_resume: Arc::new(|| {}),
        };

        PomodoroTimer::new(config, callbacks)
    }

    #[test]
    fn test_basic_workflow() -> Result<()> {
        let mut timer = create_test_timer();

        // 测试开始
        timer.start()?;
        assert_eq!(timer.get_state(), PomodoroState::Working);

        // 测试暂停
        timer.pause()?;
        assert!(matches!(timer.get_state(), PomodoroState::Paused(_)));

        // 测试继续
        timer.start()?;
        assert_eq!(timer.get_state(), PomodoroState::Working);

        // 测试停止
        timer.stop()?;
        assert_eq!(timer.get_state(), PomodoroState::Idle);

        Ok(())
    }

    #[test]
    fn test_completion() -> Result<()> {
        let mut timer = create_test_timer();
        timer.start()?;
        
        // 等待工作时间结束
        std::thread::sleep(Duration::from_secs(3));
        timer.update()?;

        let stats = timer.get_stats()?;
        assert_eq!(stats.total_completed, 1);
        assert_eq!(stats.current_streak, 1);

        Ok(())
    }

    #[test]
    fn test_interruption() -> Result<()> {
        let mut timer = create_test_timer();
        timer.start()?;
        
        // 中断
        timer.stop()?;

        let stats = timer.get_stats()?;
        assert_eq!(stats.total_interrupted, 1);
        assert_eq!(stats.current_streak, 0);

        Ok(())
    }

    #[test]
    fn test_long_break_interval() -> Result<()> {
        let mut timer = create_test_timer();

        // 完成第一个番茄钟
        timer.start()?;
        std::thread::sleep(Duration::from_secs(3));
        timer.update()?;
        assert!(matches!(timer.get_state(), PomodoroState::ShortBreak | PomodoroState::Idle));

        // 完成第二个番茄钟（应触发长休息）
        timer.start()?;
        std::thread::sleep(Duration::from_secs(3));
        timer.update()?;
        assert!(matches!(timer.get_state(), PomodoroState::LongBreak | PomodoroState::Idle));

        Ok(())
    }

    #[test]
    fn test_progress_calculation() -> Result<()> {
        let mut timer = create_test_timer();
        
        // 检查初始进度
        assert_eq!(timer.get_progress(), 0.0);

        // 启动计时器
        timer.start()?;
        std::thread::sleep(Duration::from_secs(1));
        timer.update()?;

        // 检查进度（应该接近50%）
        let progress = timer.get_progress();
        assert!(progress > 0.45 && progress < 0.55);

        Ok(())
    }

    #[test]
    fn test_auto_start_features() -> Result<()> {
        let config = PomodoroConfig {
            work_duration: Duration::from_secs(2),
            short_break_duration: Duration::from_secs(1),
            long_break_duration: Duration::from_secs(3),
            long_break_interval: 2,
            auto_start_breaks: true,
            auto_start_pomodoros: true,
            ..PomodoroConfig::default()
        };

        let mut timer = PomodoroTimer::new(config, PomodoroCallbacks::default());
        
        // 启动计时器
        timer.start()?;
        std::thread::sleep(Duration::from_secs(3));
        timer.update()?;

        // 检查是否自动开始休息
        assert_eq!(timer.get_state(), PomodoroState::ShortBreak);

        // 等待休息结束
        std::thread::sleep(Duration::from_secs(2));
        timer.update()?;

        // 检查是否自动开始下一个番茄钟
        assert_eq!(timer.get_state(), PomodoroState::Working);

        Ok(())
    }

    #[test]
    fn test_stats_tracking() -> Result<()> {
        let mut timer = create_test_timer();

        // 完成一个番茄钟
        timer.start()?;
        std::thread::sleep(Duration::from_secs(3));
        timer.update()?;

        let stats = timer.get_stats()?;
        assert_eq!(stats.total_completed, 1);
        assert_eq!(stats.total_work_time, Duration::from_secs(2));
        assert_eq!(stats.current_streak, 1);
        assert_eq!(stats.longest_streak, 1);
        assert_eq!(stats.total_interrupted, 0);
        assert_eq!(stats.daily_completed.len(), 1);

        // 中断一个番茄钟
        timer.start()?;
        std::thread::sleep(Duration::from_secs(1));
        timer.stop()?;

        let stats = timer.get_stats()?;
        assert_eq!(stats.total_completed, 1);
        assert_eq!(stats.total_interrupted, 1);
        assert_eq!(stats.current_streak, 0);

        Ok(())
    }

    #[test]
    fn test_invalid_state_transitions() {
        let mut timer = create_test_timer();

        // 未启动时尝试暂停
        assert!(timer.pause().is_err());

        // 已经运行时尝试再次启动
        timer.start().unwrap();
        assert!(timer.start().is_err());

        // 空闲时尝试更新时间
        timer.stop().unwrap();
        assert!(timer.update().is_ok()); // 应该允许但不做任何事
    }
}