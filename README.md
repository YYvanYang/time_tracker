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

### 下载安装

```bash
# macOS
brew install timetracker

# Windows
winget install timetracker

# Linux
sudo apt install timetracker
```

### 源码编译

```bash
# 安装 Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

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
| 功能 | 快捷键 |
|------|--------|
| 开始/暂停 | `Ctrl+Space` |
| 休息 | `Ctrl+B` |
| 切换视图 | `Ctrl+1-4` |

## 🔧 配置说明

### 配置文件
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
  "tracking": {
    "idle_threshold": 180
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
- 响应式 GUI
- 跨平台支持

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

### 开发流程
```bash
# 开发环境
cargo watch -x run

# 测试
cargo test

# 代码检查
cargo clippy && cargo fmt
```

### 插件开发

```rust
use timetracker_plugin::Plugin;

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
A: 调整采样间隔，关闭不需要的插件。

### 数据安全
Q: 如何备份数据？  
A: 支持本地备份和云端同步。

## 🤝 参与贡献

1. Fork 项目
2. 创建分支
3. 提交代码
4. 发起 PR

详见 [CONTRIBUTING.md](CONTRIBUTING.md)

## 📜 许可证

MIT License - 详见 [LICENSE](LICENSE)

## 🙏 致谢

- [egui](https://github.com/emilk/egui) - GUI 框架
- [rusqlite](https://github.com/rusqlite/rusqlite) - SQLite 支持
- [plotters](https://github.com/plotters-rs/plotters) - 数据可视化

## 📬 联系我们

- 作者：Your Name
- Email：your.email@example.com
- Twitter：[@yourusername](https://twitter.com/yourusername)

---
Made with ❤️ in Rust