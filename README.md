# 🌐 Chrome Tool

一个用 Rust 编写的 Chrome 浏览器版本管理器，帮助前端开发者在不同 Chrome 版本之间快速切换调试。

## ✨ 功能

- 📋 **list-remote** — 查看所有可下载的 Chrome 版本（v80 ~ 最新）
- 📦 **list** — 查看本地已安装的版本
- ⬇️ **install** — 下载并安装指定版本
- 🗑️ **uninstall** — 卸载已安装版本
- 🚀 **launch** — 启动指定版本（独立 Profile，互不干扰）

## 📥 安装

```bash
# 克隆仓库
git clone https://github.com/idiotsxp/chrome-tool.git
cd chrome-tool

# 编译
cargo build --release

# 可执行文件位于
# target/release/chrome-tool.exe
```

将 `chrome-tool.exe` 复制到 PATH 中即可全局使用。

## 🚀 使用

```bash
# 查看所有远程可用版本
chrome-tool list-remote

# 安装 Chrome 91（老版本 Chromium）
chrome-tool install 91

# 安装 Chrome 120（Chrome for Testing）
chrome-tool install 120

# 查看本地已安装版本
chrome-tool list

# 启动 Chrome 120
chrome-tool launch 120

# 启动并打开指定 URL
chrome-tool launch 120 --url https://example.com

# 卸载版本
chrome-tool uninstall 120
```

## 📊 支持版本

采用**双数据源**策略覆盖广泛版本范围：

| 版本范围 | 数据源 | 可用版本 |
|----------|--------|---------|
| v80 ~ v112 | Chromium Snapshots | 80, 83, 85, 88, 91, 95, 99, 103, 106, 109, 112 |
| v113 ~ 最新 | Chrome for Testing API | 每个 milestone 均可用 |

## 🏗️ 技术栈

- **语言**: Rust
- **CLI**: [clap](https://github.com/clap-rs/clap)
- **HTTP**: [reqwest](https://github.com/seanmonstar/reqwest) + rustls
- **解压**: [zip](https://github.com/zip-rs/zip2)
- **进度条**: [indicatif](https://github.com/console-rs/indicatif)

## 📁 本地存储

```
~/.chrome-tool/
├── versions/          # 已安装的 Chrome 版本
│   ├── 91/
│   └── 120/
├── profiles/          # 每版本独立用户数据
│   ├── 91/
│   └── 120/
└── cache/             # 下载缓存（安装后自动清理）
```

## ⚙️ 设计特点

- **独立 Profile** — 每个版本使用独立 `--user-data-dir`，多版本数据互不干扰
- **进度可视** — 下载和解压均显示进度条
- **自动清理** — 安装成功后自动删除下载缓存
- **平台适配** — 目前支持 Windows x64

## 📜 License

MIT
