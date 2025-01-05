//src/cli.rs

use crate::error::Result;
use clap::{Parser, Subcommand};
use chrono::{Local, Duration};
use std::path::PathBuf;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    /// 启用详细日志输出
    #[arg(short, long)]
    pub verbose: bool,

    /// 不显示GUI，仅在命令行中运行
    #[arg(short, long)]
    pub no_gui: bool,

    /// 命令
    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Subcommand)]
pub enum Commands {
    /// 启动番茄钟
    Start {
        /// 工作时长（分钟）
        #[arg(short, long, default_value = "25")]
        duration: u32,

        /// 项目名称
        #[arg(short, long)]
        project: Option<String>,

        /// 标签
        #[arg(short, long)]
        tags: Vec<String>,
    },

    /// 暂停番茄钟
    Pause,

    /// 停止番茄钟
    Stop,

    /// 显示统计信息
    Stats {
        /// 开始日期
        #[arg(short, long)]
        from: Option<String>,

        /// 结束日期
        #[arg(short, long)]
        to: Option<String>,

        /// 项目名称
        #[arg(short, long)]
        project: Option<String>,

        /// 输出格式 (text/json/csv)
        #[arg(short, long, default_value = "text")]
        format: String,
    },

    /// 导出数据
    Export {
        /// 输出文件路径
        #[arg(short, long)]
        output: PathBuf,

        /// 导出格式 (json/csv/excel)
        #[arg(short, long, default_value = "json")]
        format: String,

        /// 开始日期
        #[arg(short, long)]
        from: Option<String>,

        /// 结束日期
        #[arg(short, long)]
        to: Option<String>,
    },
}

pub struct CliRunner {
    storage: crate::storage::Storage,
    pomodoro: crate::pomodoro::PomodoroTimer,
}

impl CliRunner {
    pub fn new(storage: crate::storage::Storage, pomodoro: crate::pomodoro::PomodoroTimer) -> Self {
        Self {
            storage,
            pomodoro,
        }
    }

    pub async fn run(&mut self, cli: &Cli) -> Result<()> {
        match &cli.command {
            Some(Commands::Start { duration, project, tags }) => {
                self.start_pomodoro(*duration, project, tags)?;
            }
            Some(Commands::Pause) => {
                self.pause_pomodoro()?;
            }
            Some(Commands::Stop) => {
                self.stop_pomodoro()?;
            }
            Some(Commands::Stats { from, to, project, format }) => {
                self.show_stats(from, to, project, format)?;
            }
            Some(Commands::Export { output, format, from, to }) => {
                self.export_data(output, format, from, to).await?;
            }
            None => {
                if cli.no_gui {
                    // 显示当前状态
                    self.show_status()?;
                } else {
                    // 启动GUI
                    println!("Starting GUI...");
                }
            }
        }

        Ok(())
    }

    fn start_pomodoro(
        &mut self,
        duration: u32,
        project: &Option<String>,
        tags: &[String],
    ) -> Result<()> {
        // 设置番茄钟时长
        self.pomodoro.set_work_duration(Duration::minutes(duration as i64));

        // 设置项目和标签
        if let Some(project_name) = project {
            if let Ok(project) = self.storage.get_project_by_name(project_name) {
                self.pomodoro.set_project(project.id);
            } else {
                println!("警告: 项目 '{}' 不存在", project_name);
            }
        }
        self.pomodoro.set_tags(tags.to_vec());

        // 启动番茄钟
        self.pomodoro.start()?;
        println!("番茄钟已启动 ({} 分钟)", duration);

        // 监听状态变化
        let mut last_remaining = self.pomodoro.get_remaining_time();
        while self.pomodoro.is_active() {
            std::thread::sleep(std::time::Duration::from_secs(1));
            let remaining = self.pomodoro.get_remaining_time();
            if remaining != last_remaining {
                print!("\r剩余时间: {:02}:{:02}   ",
                    remaining.as_secs() / 60,
                    remaining.as_secs() % 60
                );
                std::io::Write::flush(&mut std::io::stdout())?;
                last_remaining = remaining;
            }
        }

        println!("\n番茄钟完成！");
        Ok(())
    }

    fn pause_pomodoro(&mut self) -> Result<()> {
        self.pomodoro.pause()?;
        println!("番茄钟已暂停");
        Ok(())
    }

    fn stop_pomodoro(&mut self) -> Result<()> {
        self.pomodoro.stop()?;
        println!("番茄钟已停止");
        Ok(())
    }

    fn show_stats(
        &self,
        from: &Option<String>,
        to: &Option<String>,
        project: &Option<String>,
        format: &str,
    ) -> Result<()> {
        let start_date = if let Some(date) = from {
            chrono::NaiveDate::parse_from_str(date, "%Y-%m-%d")
                .map_err(|e| crate::error::TimeTrackerError::Parse(e.to_string()))?
                .and_hms_opt(0, 0, 0)
                .ok_or_else(|| crate::error::TimeTrackerError::Parse("Invalid time".into()))?
                .and_local_timezone(Local)
                .single()
                .ok_or_else(|| crate::error::TimeTrackerError::Parse("Invalid timezone".into()))?
        } else {
            Local::now() - Duration::days(30)
        };

        let end_date = if let Some(date) = to {
            chrono::NaiveDate::parse_from_str(date, "%Y-%m-%d")?
                .and_hms_opt(23, 59, 59)
                .unwrap()
                .and_local_timezone(Local)
                .unwrap()
        } else {
            Local::now()
        };

        let export_format = match format.to_lowercase().as_str() {
            "json" => crate::export::ExportFormat::JSON,
            "csv" => crate::export::ExportFormat::CSV,
            "excel" => crate::export::ExportFormat::Excel,
            _ => return Err(crate::error::TimeTrackerError::Platform(
                format!("Unsupported export format: {}", format)
            )),
        };

        let exporter = crate::export::DataExporter::new(export_format)
            .start_date(start_date)
            .end_date(end_date);

        println!("正在导出数据...");
        exporter.export(&self.storage, output)?;
        println!("数据已导出到: {}", output.display());

        Ok(())
    }

    fn show_status(&self) -> Result<()> {
        // 显示番茄钟状态
        match self.pomodoro.get_state() {
            crate::pomodoro::PomodoroState::Working => {
                let remaining = self.pomodoro.get_remaining_time();
                println!("状态: 工作中");
                println!("剩余时间: {:02}:{:02}",
                    remaining.as_secs() / 60,
                    remaining.as_secs() % 60
                );

                if let Some(project_id) = self.pomodoro.get_project() {
                    if let Ok(project) = self.storage.get_project(project_id) {
                        println!("当前项目: {}", project.name);
                    }
                }

                let tags = self.pomodoro.get_tags();
                if !tags.is_empty() {
                    println!("标签: {}", tags.join(", "));
                }
            }
            crate::pomodoro::PomodoroState::ShortBreak => {
                let remaining = self.pomodoro.get_remaining_time();
                println!("状态: 短休息");
                println!("剩余时间: {:02}:{:02}",
                    remaining.as_secs() / 60,
                    remaining.as_secs() % 60
                );
            }
            crate::pomodoro::PomodoroState::LongBreak => {
                let remaining = self.pomodoro.get_remaining_time();
                println!("状态: 长休息");
                println!("剩余时间: {:02}:{:02}",
                    remaining.as_secs() / 60,
                    remaining.as_secs() % 60
                );
            }
            crate::pomodoro::PomodoroState::Paused(_) => {
                println!("状态: 已暂停");
            }
            _ => {
                println!("状态: 空闲");
            }
        }

        // 显示今日统计
        let today = Local::now().date_naive();
        if let Ok(stats) = self.storage.get_daily_stats(today) {
            println!("\n今日统计:");
            println!("工作时长: {:.1} 小时", stats.total_work_time.as_secs_f64() / 3600.0);
            println!("完成番茄数: {}", stats.completed_pomodoros);
            println!("中断番茄数: {}", stats.interrupted_pomodoros);
            if let Some(app) = stats.most_used_app {
                println!("最常用应用: {}", app);
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_cli_start_pomodoro() {
        let temp_dir = TempDir::new().unwrap();
        let storage = crate::storage::Storage::new(&temp_dir.path().to_path_buf()).unwrap();
        let pomodoro = crate::pomodoro::PomodoroTimer::new(Default::default(), Default::default());
        let mut runner = CliRunner::new(storage, pomodoro);

        // 测试启动番茄钟
        runner.start_pomodoro(1, &None, &Vec::new()).unwrap();
        assert!(matches!(
            runner.pomodoro.get_state(),
            crate::pomodoro::PomodoroState::Working
        ));
    }

    #[tokio::test]
    async fn test_cli_export() {
        let temp_dir = TempDir::new().unwrap();
        let storage = crate::storage::Storage::new(&temp_dir.path().to_path_buf()).unwrap();
        let pomodoro = crate::pomodoro::PomodoroTimer::new(Default::default(), Default::default());
        let mut runner = CliRunner::new(storage, pomodoro);

        let output = temp_dir.path().join("export.json");
        
        // 测试导出数据
        runner.export_data(
            &output,
            "json",
            &Some(Local::now().format("%Y-%m-%d").to_string()),
            &None,
        ).await.unwrap();

        assert!(output.exists());
    }

    #[test]
    fn test_cli_stats() {
        let temp_dir = TempDir::new().unwrap();
        let storage = crate::storage::Storage::new(&temp_dir.path().to_path_buf()).unwrap();
        let pomodoro = crate::pomodoro::PomodoroTimer::new(Default::default(), Default::default());
        let mut runner = CliRunner::new(storage, pomodoro);

        // 测试显示统计信息
        runner.show_stats(
            &Some(Local::now().format("%Y-%m-%d").to_string()),
            &None,
            &None,
            "text",
        ).unwrap();
    }
}

// 添加main.rs中的CLI支持
#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    // 设置日志级别
    if cli.verbose {
        std::env::set_var("RUST_LOG", "debug");
    } else {
        std::env::set_var("RUST_LOG", "info");
    }
    env_logger::init();

    // 初始化存储和番茄钟
    let config = crate::config::Config::load()?;
    let storage = crate::storage::Storage::new(&config.storage.data_dir)?;
    let pomodoro = crate::pomodoro::PomodoroTimer::new(
        config.pomodoro.clone(),
        Default::default(),
    );

    // 如果不是命令行模式，启动GUI
    if !cli.no_gui {
        return crate::ui::run_gui(config, storage, pomodoro);
    }

    // 运行CLI命令
    let mut runner = CliRunner::new(storage, pomodoro);
    runner.run(&cli).await?;

    Ok(())
}