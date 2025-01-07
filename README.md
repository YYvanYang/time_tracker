# Time Tracker

一个跨平台的工作时间追踪器，帮助你专注工作、提高效率。

[![Build Status](https://github.com/yourusername/timetracker/workflows/CI/badge.svg)](https://github.com/yourusername/timetracker/actions)
[![Latest Release](https://img.shields.io/github/v/release/yourusername/timetracker)](https://github.com/yourusername/timetracker/releases)
[![License](https://img.shields.io/github/license/yourusername/timetracker)](LICENSE)

![Screenshot](docs/images/screenshot.png)

## 特性

### 🎯 工作时间追踪
- 自动记录应用使用时间
- 分类统计工作效率
- 数据可视化展示
- 每日/周/月报告

### 🍅 番茄工作法
- 可自定义工作和休息时长
- 智能休息提醒
- 项目和标签管理
- 进度统计分析

### 📊 数据分析
- 生产力趋势分析
- 工作模式识别
- 智能建议
- 数据导出功能

### 🔧 系统功能
- 系统托盘支持
- 快捷键操作
- 自动更新
- 多语言支持
- 主题切换

### 🔌 插件系统
- 可扩展的插件架构
- 自定义数据分析插件
- 第三方服务集成
- 自定义导出格式

## 安装

### Windows
1. 从 [Releases](https://github.com/yourusername/timetracker/releases) 下载最新的安装包
2. 运行安装程序
3. 根据提示完成安装

### macOS
```bash
brew install timetracker
```

### Linux
```bash
# Ubuntu/Debian
sudo apt install timetracker

# Arch Linux
yay -S timetracker
```

### 从源码编译
```bash
# 安装 Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# 安装系统依赖
## macOS
brew install sqlite3

## Ubuntu/Debian
sudo apt-get update
sudo apt-get install -y libsqlite3-dev libssl-dev pkg-config

# 克隆仓库
git clone https://github.com/yourusername/timetracker.git
cd timetracker

# 安装开发工具
cargo install cargo-watch cargo-audit cargo-make

# 编译
cargo build --release

# 安装
cargo install --path .
```

## 使用方法

### 图形界面
1. 启动程序后，它会自动开始追踪你的工作时间
2. 点击托盘图标可以快速访问常用功能
3. 主界面包含以下功能区：
   - 概览：显示当前工作状态和统计数据
   - 应用统计：查看应用使用时间分布
   - 番茄钟：管理番茄工作法会话
   - 项目：管理和追踪项目进度
   - 统计：查看详细的统计报告

### 命令行界面
```bash
# 显示帮助
timetracker --help

# 启动番茄钟会话
timetracker start --duration 25 --project "My Project"

# 查看统计信息
timetracker stats --from 2024-01-01 --to 2024-01-31

# 导出数据
timetracker export --format json --output timetracker.json
```

### 快捷键
- `Ctrl+Space`: 开始/暂停番茄钟
- `Ctrl+B`: 开始休息
- `Ctrl+1`: 切换到概览
- `Ctrl+2`: 切换到应用统计
- `Ctrl+3`: 切换到番茄钟
- `Ctrl+Q`: 退出程序

## 配置

配置文件位于：
- Windows: `%APPDATA%\TimeTracker\config.json`
- macOS: `~/Library/Application Support/TimeTracker/config.json`
- Linux: `~/.config/timetracker/config.json`

```json
{
  "general": {
    "autostart": true,
    "language": "zh-CN",
    "minimize_to_tray": true
  },
  "pomodoro": {
    "work_duration": 1500,
    "short_break_duration": 300,
    "long_break_duration": 900,
    "long_break_interval": 4
  }
}
```

## 数据存储

数据存储位置：
- Windows: `%APPDATA%\TimeTracker\data`
- macOS: `~/Library/Application Support/TimeTracker/data`
- Linux: `~/.local/share/timetracker/data`

支持自动备份和数据导出：
- 自动备份：每天创建一次备份
- 数据导出：支持 JSON、CSV、Excel 格式
- 数据同步：支持云端备份（需要设置）

## 开发

### 环境要求
- Rust 1.70.0 或更高版本
- Cargo
- 系统依赖：
  - Windows: MSVC
  - macOS: Xcode Command Line Tools
  - Linux: gcc, pkg-config, gtk3-devel

### 开发流程
```bash
# 安装开发依赖
cargo install cargo-watch cargo-audit

# 运行测试
cargo test

# 开发模式运行
cargo run

# 构建发布版本
cargo build --release

# 运行 lint
cargo clippy

# 格式化代码
cargo fmt
```

### 项目结构
```
src/
├── main.rs           # 程序入口
├── lib.rs            # 库入口
├── core/             # 核心模块
│   ├── models.rs     # 核心模型
│   ├── traits.rs     # 核心特征
│   └── error.rs      # 错误处理
├── domain/           # 领域层
│   ├── activity.rs   # 活动领域
│   ├── pomodoro.rs   # 番茄钟领域
│   ├── project.rs    # 项目领域
│   ├── plugin.rs     # 插件领域
│   └── analysis.rs   # 分析领域
├── application/      # 应用层
├── infrastructure/   # 基础设施层
│   ├── platform/     # 平台特定实现
│   ├── config.rs     # 配置管理
│   └── logging.rs    # 日志系统
├── plugins/          # 插件系统
└── presentation/     # 展示层
```

项目采用领域驱动设计(DDD)架构，通过清晰的分层设计提供更好的可维护性和扩展性。

## 技术架构

### 核心特性
- 领域驱动设计(DDD)架构
- 插件化系统设计
- 跨平台支持（Windows/macOS）
- 模块化的代码组织

### 技术栈
- 语言：Rust
- GUI：egui
- 存储：SQLite
- 跨平台：特定平台API抽象

## 贡献指南

欢迎贡献代码！请查看 [CONTRIBUTING.md](CONTRIBUTING.md) 了解详情。

1. Fork 项目
2. 创建特性分支
3. 提交改动
4. 推送到分支
5. 创建 Pull Request

## 路线图

- [x] 基础时间追踪
- [x] 番茄工作法
- [x] 项目管理
- [x] 数据分析
- [x] 多语言支持
- [ ] 团队协作功能
- [ ] 移动端应用
- [ ] 插件系统

## 常见问题

### Q: 程序无法自动启动？
A: 请检查系统设置中的自启动权限。

### Q: 统计数据不准确？
A: 请确保程序拥有必要的系统权限来追踪应用使用情况。

### Q: 如何恢复备份？
A: 在设置中选择"恢复备份"，然后选择备份文件即可。

## 许可证

本项目采用 MIT 许可证 - 查看 [LICENSE](LICENSE) 文件了解详情。

## 致谢

- [egui](https://github.com/emilk/egui) - GUI 框架
- [rusqlite](https://github.com/rusqlite/rusqlite) - SQLite 绑定
- [plotters](https://github.com/plotters-rs/plotters) - 数据可视化
- [其他依赖库的作者们](ACKNOWLEDGMENTS.md)

## 联系方式

- 作者：Your Name
- Email：your.email@example.com
- Twitter：[@yourusername](https://twitter.com/yourusername)

如果你发现了 bug 或有新功能建议，请[创建 Issue](https://github.com/yourusername/timetracker/issues)。

---
Made with ❤️ in Rust

## 插件开发

TimeTracker 提供了强大的插件系统，允许开发者扩展和定制功能。

### 插件类型
- 数据分析插件：自定义数据分析和可视化
- 导出插件：支持自定义导出格式
- 集成插件：与第三方服务集成
- 界面插件：自定义UI组件

### 创建插件
1. 创建新的 Rust 项目
2. 添加依赖
```toml
[dependencies]
timetracker-plugin = { git = "https://github.com/yourusername/timetracker" }
```

3. 实现插件特征
```rust
use timetracker_plugin::Plugin;

#[derive(Default)]
struct MyPlugin;

impl Plugin for MyPlugin {
    fn name(&self) -> &str {
        "my_plugin"
    }

    fn on_load(&self) -> Result<(), Box<dyn Error>> {
        // 插件初始化逻辑
        Ok(())
    }
}
```

4. 构建和安装
```bash
cargo build --release
cp target/release/libmy_plugin.* ~/.timetracker/plugins/
```

### 插件配置
在配置文件中启用插件：
```json
{
  "plugins": {
    "enabled": ["my_plugin"],
    "settings": {
      "my_plugin": {
        "option1": "value1"
      }
    }
  }
}
```

## 开发环境设置

### 推荐的开发工具
- VS Code 或 RustRover
- rust-analyzer 插件
- LLDB 调试器
- SQLite 浏览器

### 代码风格
- 遵循 Rust 标准代码风格
- 使用 rustfmt 格式化代码
- 使用 clippy 进行代码检查
- 编写单元测试和集成测试

### 调试技巧
```bash
# 启用详细日志
RUST_LOG=debug cargo run

# 运行特定测试
cargo test test_name -- --nocapture

# 性能分析
cargo install flamegraph
cargo flamegraph
```