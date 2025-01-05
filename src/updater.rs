//src/updater.rs

use crate::error::{Result, TimeTrackerError};
use serde::Deserialize;
use semver::Version;
use std::path::PathBuf;
use tokio;
use reqwest;

const GITHUB_API_URL: &str = "https://api.github.com/repos/yourusername/timetracker/releases/latest";
const UPDATE_CHECK_INTERVAL: std::time::Duration = std::time::Duration::from_secs(60 * 60 * 24); // 每天检查一次

#[derive(Debug, Deserialize)]
struct GithubRelease {
    tag_name: String,
    body: String,
    assets: Vec<GithubAsset>,
    html_url: String,
}

#[derive(Debug, Deserialize)]
struct GithubAsset {
    name: String,
    browser_download_url: String,
    size: u64,
}

#[derive(Debug)]
pub struct UpdateInfo {
    pub version: Version,
    pub release_notes: String,
    pub download_url: String,
    pub file_size: u64,
    pub release_page: String,
}

pub struct Updater {
    current_version: Version,
    last_check: std::time::Instant,
    download_dir: PathBuf,
    client: reqwest::Client,
}

impl Updater {
    pub fn new() -> Result<Self> {
        let current_version = Version::parse(env!("CARGO_PKG_VERSION"))?;
        let download_dir = dirs::download_dir()
            .ok_or_else(|| TimeTrackerError::Platform("Cannot find downloads directory".into()))?
            .join("TimeTracker");

        std::fs::create_dir_all(&download_dir)?;

        Ok(Self {
            current_version,
            last_check: std::time::Instant::now(),
            download_dir,
            client: reqwest::Client::new(),
        })
    }

    pub async fn check_update(&mut self) -> Result<Option<UpdateInfo>> {
        // 检查是否需要更新
        if self.last_check.elapsed() < UPDATE_CHECK_INTERVAL {
            return Ok(None);
        }
        self.last_check = std::time::Instant::now();

        // 获取最新版本信息
        let release = self.fetch_latest_release().await?;
        let latest_version = Version::parse(&release.tag_name.trim_start_matches('v'))?;

        // 比较版本
        if latest_version <= self.current_version {
            return Ok(None);
        }

        // 找到对应平台的下载包
        let asset = self.find_platform_asset(&release.assets)?;

        Ok(Some(UpdateInfo {
            version: latest_version,
            release_notes: release.body,
            download_url: asset.browser_download_url,
            file_size: asset.size,
            release_page: release.html_url,
        }))
    }

    async fn fetch_latest_release(&self) -> Result<GithubRelease> {
        let response = self.client
            .get(GITHUB_API_URL)
            .header("User-Agent", "TimeTracker")
            .send()
            .await
            .map_err(|e| TimeTrackerError::Platform(format!("Failed to fetch release info: {}", e)))?;

        if !response.status().is_success() {
            return Err(TimeTrackerError::Platform(
                format!("Failed to fetch release info: {}", response.status())
            ));
        }

        let release = response.json::<GithubRelease>()
            .await
            .map_err(|e| TimeTrackerError::Platform(format!("Invalid release info: {}", e)))?;

        Ok(release)
    }

    fn find_platform_asset(&self, assets: &[GithubAsset]) -> Result<&GithubAsset> {
        let platform_suffix = if cfg!(target_os = "windows") {
            "-windows-x86_64.exe"
        } else if cfg!(target_os = "macos") {
            "-macos-x86_64.dmg"
        } else if cfg!(target_os = "linux") {
            "-linux-x86_64.AppImage"
        } else {
            return Err(TimeTrackerError::Platform("Unsupported platform".into()));
        };

        assets.iter()
            .find(|asset| asset.name.ends_with(platform_suffix))
            .ok_or_else(|| TimeTrackerError::Platform(
                format!("No release found for platform: {}", platform_suffix)
            ))
    }

    pub async fn download_update(&self, info: &UpdateInfo) -> Result<PathBuf> {
        let filename = self.download_dir.join(
            info.download_url
                .split('/')
                .last()
                .ok_or_else(|| TimeTrackerError::Platform("Invalid download URL".into()))?
        );

        // 下载文件
        let response = self.client
            .get(&info.download_url)
            .send()
            .await
            .map_err(|e| TimeTrackerError::Platform(format!("Failed to download update: {}", e)))?;

        let mut file = tokio::fs::File::create(&filename)
            .await
            .map_err(|e| TimeTrackerError::Platform(format!("Failed to create file: {}", e)))?;

        let mut stream = response.bytes_stream();
        while let Some(chunk) = stream.next().await {
            let chunk = chunk.map_err(|e| {
                TimeTrackerError::Platform(format!("Failed to download chunk: {}", e))
            })?;
            tokio::io::copy(&mut chunk.as_ref(), &mut file).await
                .map_err(|e| TimeTrackerError::Platform(format!("Failed to write chunk: {}", e)))?;
        }

        Ok(filename)
    }

    pub fn install_update(&self, file_path: PathBuf) -> Result<()> {
        #[cfg(target_os = "windows")]
        {
            use std::process::Command;
            Command::new(file_path)
                .arg("/SILENT")
                .spawn()
                .map_err(|e| TimeTrackerError::Platform(
                    format!("Failed to start installer: {}", e)
                ))?;
        }

        #[cfg(target_os = "macos")]
        {
            use std::process::Command;
            Command::new("open")
                .arg(file_path)
                .spawn()
                .map_err(|e| TimeTrackerError::Platform(
                    format!("Failed to open installer: {}", e)
                ))?;
        }

        #[cfg(target_os = "linux")]
        {
            use std::process::Command;
            Command::new(file_path)
                .spawn()
                .map_err(|e| TimeTrackerError::Platform(
                    format!("Failed to start AppImage: {}", e)
                ))?;
        }

        Ok(())
    }
}

// 更新对话框
pub struct UpdateDialog {
    update_info: UpdateInfo,
    downloading: bool,
    download_progress: f32,
    error: Option<String>,
}

impl UpdateDialog {
    pub fn new(update_info: UpdateInfo) -> Self {
        Self {
            update_info,
            downloading: false,
            download_progress: 0.0,
            error: None,
        }
    }

    pub fn show(&mut self, ui: &mut eframe::egui::Ui) -> bool {
        use eframe::egui;

        let mut close = false;
        
        ui.heading("发现新版本");
        ui.label(format!("新版本: v{}", self.update_info.version));
        
        ui.separator();
        
        // 更新说明
        ui.heading("更新内容");
        ui.label(&self.update_info.release_notes);
        
        ui.separator();
        
        // 文件信息
        ui.label(format!(
            "文件大小: {:.1} MB",
            self.update_info.file_size as f64 / 1024.0 / 1024.0
        ));
        
        if let Some(error) = &self.error {
            ui.colored_label(ui.style().visuals.error_fg_color, error);
        }
        
        ui.separator();
        
        // 下载进度
        if self.downloading {
            ui.add(egui::ProgressBar::new(self.download_progress));
        }
        
        ui.horizontal(|ui| {
            if ui.button("查看发布页面").clicked() {
                if let Err(e) = open::that(&self.update_info.release_page) {
                    self.error = Some(format!("无法打开浏览器: {}", e));
                }
            }
            
            if !self.downloading {
                if ui.button("立即更新").clicked() {
                                        self.downloading = true;
                }
            }
            
            if ui.button("稍后再说").clicked() {
                close = true;
            }
        });
        
        close
    }

    pub fn set_progress(&mut self, progress: f32) {
        self.download_progress = progress;
    }

    pub fn set_error(&mut self, error: String) {
        self.error = Some(error);
        self.downloading = false;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::runtime::Runtime;
    use mockito::mock;

    #[test]
    fn test_version_comparison() {
        let rt = Runtime::new().unwrap();
        rt.block_on(async {
            let mut updater = Updater::new().unwrap();
            
            // Mock the GitHub API response
            let mock = mock("GET", "/repos/yourusername/timetracker/releases/latest")
                .with_status(200)
                .with_header("content-type", "application/json")
                .with_body(r#"{
                    "tag_name": "v999.0.0",
                    "body": "Test release",
                    "html_url": "https://github.com/test",
                    "assets": [{
                        "name": "timetracker-windows-x86_64.exe",
                        "browser_download_url": "https://test/download",
                        "size": 1000000
                    }]
                }"#)
                .create();

            let result = updater.check_update().await.unwrap();
            assert!(result.is_some());
            let info = result.unwrap();
            assert!(info.version > updater.current_version);

            mock.assert();
        });
    }

    #[test]
    fn test_platform_detection() {
        let updater = Updater::new().unwrap();
        let assets = vec![
            GithubAsset {
                name: "timetracker-windows-x86_64.exe".to_string(),
                browser_download_url: "https://test/windows".to_string(),
                size: 1000000,
            },
            GithubAsset {
                name: "timetracker-macos-x86_64.dmg".to_string(),
                browser_download_url: "https://test/macos".to_string(),
                size: 1000000,
            },
            GithubAsset {
                name: "timetracker-linux-x86_64.AppImage".to_string(),
                browser_download_url: "https://test/linux".to_string(),
                size: 1000000,
            },
        ];

        let asset = updater.find_platform_asset(&assets).unwrap();
        if cfg!(target_os = "windows") {
            assert!(asset.name.ends_with("windows-x86_64.exe"));
        } else if cfg!(target_os = "macos") {
            assert!(asset.name.ends_with("macos-x86_64.dmg"));
        } else if cfg!(target_os = "linux") {
            assert!(asset.name.ends_with("linux-x86_64.AppImage"));
        }
    }
}

// 定义更新流程的状态机
#[derive(Debug, Clone, PartialEq)]
pub enum UpdateState {
    Idle,
    Checking,
    UpdateAvailable(UpdateInfo),
    Downloading { progress: f32 },
    Downloaded { path: PathBuf },
    Installing,
    Error(String),
}

// 更新管理器
pub struct UpdateManager {
    updater: Updater,
    state: UpdateState,
    auto_check: bool,
    last_check: Option<std::time::SystemTime>,
}

impl UpdateManager {
    pub fn new() -> Result<Self> {
        Ok(Self {
            updater: Updater::new()?,
            state: UpdateState::Idle,
            auto_check: true,
            last_check: None,
        })
    }

    pub async fn check_for_updates(&mut self) -> Result<()> {
        self.state = UpdateState::Checking;

        match self.updater.check_update().await {
            Ok(Some(info)) => {
                self.state = UpdateState::UpdateAvailable(info);
            }
            Ok(None) => {
                self.state = UpdateState::Idle;
            }
            Err(e) => {
                self.state = UpdateState::Error(e.to_string());
            }
        }

        self.last_check = Some(std::time::SystemTime::now());
        Ok(())
    }

    pub async fn download_update(&mut self, info: UpdateInfo) -> Result<()> {
        self.state = UpdateState::Downloading { progress: 0.0 };

        match self.updater.download_update(&info).await {
            Ok(path) => {
                self.state = UpdateState::Downloaded { path };
            }
            Err(e) => {
                self.state = UpdateState::Error(e.to_string());
            }
        }

        Ok(())
    }

    pub fn install_update(&mut self, path: PathBuf) -> Result<()> {
        self.state = UpdateState::Installing;

        if let Err(e) = self.updater.install_update(path) {
            self.state = UpdateState::Error(e.to_string());
            return Err(e);
        }

        Ok(())
    }

    pub fn get_state(&self) -> &UpdateState {
        &self.state
    }

    pub fn set_auto_check(&mut self, auto_check: bool) {
        self.auto_check = auto_check;
    }

    pub fn should_check(&self) -> bool {
        if !self.auto_check {
            return false;
        }

        if let Some(last_check) = self.last_check {
            if last_check.elapsed().unwrap_or_default() < UPDATE_CHECK_INTERVAL {
                return false;
            }
        }

        matches!(self.state, UpdateState::Idle)
    }
}