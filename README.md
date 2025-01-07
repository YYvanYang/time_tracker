# Time Tracker

> 基于 Rust 开发的高效工作时间追踪器，助你提升工作效率，实现目标管理。

[![Build Status](https://github.com/yourusername/timetracker/workflows/CI/badge.svg)](https://github.com/yourusername/timetracker/actions)
[![Latest Release](https://img.shields.io/github/v/release/yourusername/timetracker)](https://github.com/yourusername/timetracker/releases)
[![License](https://img.shields.io/github/license/yourusername/timetracker)](LICENSE)

![Screenshot](docs/images/screenshot.png)

## 📚 目录

- [功能特性](#功能特性)
- [快速开始](#快速开始)
- [使用指南](#使用指南)
- [开发文档](#开发文档)
- [常见问题](#常见问题)

## 🌟 功能特性

### 🎯 智能时间追踪
- 自动记录应用使用情况
- 智能识别工作状态
- 多维度数据统计
- 深度洞察工作模式

### 🍅 专注管理
- 番茄工作法支持
- 智能休息提醒
- 自定义工作周期
- 专注度分析

### 📊 数据分析
- 可视化报告
- AI 辅助分析
- 多维度统计
- 数据导出

### 🔌 扩展能力
- 插件系统
- 主题定制
- 多语言支持
- 第三方集成

## 🚀 快速开始

### 系统要求
- Rust 1.70.0 或更高版本
- SQLite 3.x
- 支持的操作系统：
  - Windows 10/11
  - macOS 10.15+
  - Linux (主流发行版)

### 下载安装

```bash
# macOS
brew install timetracker

# Windows
winget install timetracker

# Linux
sudo apt install timetracker  # Ubuntu/Debian
yay -S timetracker           # Arch Linux
```

### 源码编译

```bash
# 安装 Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# 安装系统依赖
## macOS
brew install sqlite3

## Ubuntu/Debian
sudo apt-get update
sudo apt-get install -y libsqlite3-dev pkg-config

# 克隆编译
git clone https://github.com/yourusername/timetracker.git
cd timetracker
cargo install --path .
```

## 📖 使用指南

### 基础操作
- 启动后自动追踪工作时间
- 系统托盘快速访问
- 快捷键控制

### 界面功能
- 仪表盘：工作概览
- 时间线：活动记录
- 番茄钟：专注管理
- 统计：数据分析

### 快捷键
| 功能 | Windows/Linux | macOS |
|------|--------------|-------|
| 开始/暂停 | `Ctrl+Space` | `⌘+Space` |
| 休息 | `Ctrl+B` | `⌘+B` |
| 切换视图 | `Ctrl+1-4` | `⌘+1-4` |

## 🔧 配置说明

### 配置文件位置
```
Windows: %APPDATA%\TimeTracker\config.json
macOS:   ~/Library/Application Support/TimeTracker/config.json
Linux:   ~/.config/timetracker/config.json
```

### 示例配置
```json
{
  "general": {
    "language": "zh-CN",
    "theme": "auto"
  },
  "database": {
    "url": "sqlite://timetracker.db",
    "pool_size": 5
  },
  "tracking": {
    "idle_threshold": 180,
    "sync_interval": 60
  },
  "pomodoro": {
    "work_duration": 25,
    "break_duration": 5
  }
}
```

## 💻 开发文档

### 技术架构
- DDD 架构设计
- 插件化系统
- 异步运行时
- 跨平台支持

### 技术栈
- 核心：Rust + Tokio 异步运行时
- GUI：eframe (egui framework)
- 数据库：SQLx + SQLite
- 系统集成：tray-item

### 目录结构
```
src/
├── core/          # 核心模型
├── domain/        # 业务逻辑
├── application/   # 应用服务
├── infrastructure/# 基础设施
├── plugins/       # 插件系统
└── presentation/  # 界面层
```

### 开发环境设置
```bash
# 安装开发工具
cargo install cargo-watch cargo-audit sqlx-cli

# 数据库迁移
sqlx database create
sqlx migrate run

# 开发模式运行
cargo watch -x run

# 测试
cargo test

# 代码检查
cargo clippy && cargo fmt
```

### 插件开发

```rust
use time_tracker_plugin::Plugin;
use anyhow::Result;

#[derive(Default)]
pub struct MyPlugin;

impl Plugin for MyPlugin {
    fn name(&self) -> &str { "my_plugin" }
    fn on_load(&self) -> Result<()> { Ok(()) }
}
```

## ❓ 常见问题

### 性能相关
Q: CPU 占用较高？  
A: 调整配置中的 `sync_interval` 和 `idle_threshold` 参数。

### 数据安全
Q: 如何备份数据？  
A: 数据库文件位于配置目录中，可以直接备份该文件。

## 🤝 参与贡献

1. Fork 项目
2. 创建分支
3. 提交代码
4. 发起 PR

详见 [CONTRIBUTING.md](CONTRIBUTING.md)

## 📜 许可证

MIT License - 详见 [LICENSE](LICENSE)

## 🙏 致谢

- [eframe](https://github.com/emilk/egui) - GUI 框架
- [SQLx](https://github.com/launchbadge/sqlx) - 异步 SQL 工具包
- [Tokio](https://github.com/tokio-rs/tokio) - 异步运行时
- [tray-item](https://github.com/olback/tray-item-rs) - 系统托盘支持

## 📬 联系我们

- 作者：Your Name
- Email：your.email@example.com
- Twitter：[@yourusername](https://twitter.com/yourusername)

---
Made with ❤️ in Rust