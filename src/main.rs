use time_tracker::core::AppResult;
use time_tracker::infrastructure::storage::Storage;

#[tokio::main]
async fn main() -> AppResult<()> {
    // 初始化日志
    env_logger::init();

    // 获取数据目录
    let data_dir = dirs::data_dir()
        .unwrap_or_else(|| std::path::PathBuf::from("."))
        .join("time_tracker");

    // 确保数据目录存在
    std::fs::create_dir_all(&data_dir)?;

    // 初始化存储
    let database_path = data_dir.join("timetracker.db");
    Storage::initialize(database_path).await?;

    // TODO: 初始化其他组件并启动应用程序

    Ok(())
}