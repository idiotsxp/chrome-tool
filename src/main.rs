mod api;
mod download;
mod launcher;
mod models;
mod storage;

use anyhow::Result;
use clap::{Parser, Subcommand};
use colored::*;

use models::VersionSource;

#[derive(Parser)]
#[command(
    name = "chrome-tool",
    about = "Chrome 浏览器版本管理工具 - 轻松切换不同版本的 Chrome",
    version
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// 列出远程可下载的 Chrome 版本
    #[command(name = "list-remote")]
    ListRemote,

    /// 列出本地已安装的 Chrome 版本
    #[command(name = "list")]
    List,

    /// 安装指定 milestone 版本的 Chrome
    #[command(name = "install")]
    Install {
        /// Chrome milestone 版本号 (例如: 80, 91, 120, 130)
        milestone: u32,
    },

    /// 卸载指定版本
    #[command(name = "uninstall")]
    Uninstall {
        /// Chrome milestone 版本号
        milestone: u32,
    },

    /// 启动指定版本的 Chrome
    #[command(name = "launch")]
    Launch {
        /// Chrome milestone 版本号
        milestone: u32,

        /// 启动时打开的 URL
        #[arg(long)]
        url: Option<String>,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::ListRemote => cmd_list_remote().await?,
        Commands::List => cmd_list()?,
        Commands::Install { milestone } => cmd_install(milestone).await?,
        Commands::Uninstall { milestone } => cmd_uninstall(milestone)?,
        Commands::Launch { milestone, url } => cmd_launch(milestone, url)?,
    }

    Ok(())
}

async fn cmd_list_remote() -> Result<()> {
    println!("{}", "\n🌐 正在获取远程 Chrome 版本列表...\n".cyan().bold());

    let versions = api::fetch_all_versions().await?;

    // Print header
    println!(
        "  {:<12} {:<22} {}",
        "Milestone".bold(),
        "Version".bold(),
        "Source".bold()
    );
    println!("  {}", "─".repeat(60));

    for v in &versions {
        let source_tag = match v.source {
            VersionSource::ChromeForTesting => "Chrome for Testing".green(),
            VersionSource::ChromiumSnapshot => "Chromium Snapshot".yellow(),
        };

        // Check if already installed
        let installed = storage::is_installed(v.milestone).unwrap_or(false);
        let marker = if installed {
            " ✓".green().to_string()
        } else {
            String::new()
        };

        println!(
            "  {:<12} {:<22} {}{}",
            format!("{}", v.milestone).white().bold(),
            v.version,
            source_tag,
            marker
        );
    }

    println!(
        "\n  共 {} 个可用版本\n",
        versions.len().to_string().cyan()
    );

    Ok(())
}

fn cmd_list() -> Result<()> {
    println!("{}", "\n📦 本地已安装的 Chrome 版本:\n".cyan().bold());

    let installed = storage::list_installed()?;

    if installed.is_empty() {
        println!("  {}", "暂无已安装版本".dimmed());
        println!(
            "\n  使用 {} 安装版本\n",
            "chrome-tool install <milestone>".green()
        );
        return Ok(());
    }

    println!(
        "  {:<12} {}",
        "Milestone".bold(),
        "Chrome 路径".bold()
    );
    println!("  {}", "─".repeat(70));

    for v in &installed {
        println!(
            "  {:<12} {}",
            format!("{}", v.milestone).white().bold(),
            v.chrome_exe.display().to_string().dimmed()
        );
    }

    println!(
        "\n  共 {} 个已安装版本\n",
        installed.len().to_string().cyan()
    );

    Ok(())
}

async fn cmd_install(milestone: u32) -> Result<()> {
    println!(
        "{}",
        format!("\n⬇️  安装 Chrome {}...\n", milestone).cyan().bold()
    );

    // Check if already installed
    if storage::is_installed(milestone)? {
        println!("  {} 版本 {} 已安装", "✓".green(), milestone);
        if let Ok(Some(exe)) = storage::find_chrome_exe(milestone) {
            println!("  路径: {}", exe.display().to_string().dimmed());
        }
        println!();
        return Ok(());
    }

    // Get all available versions
    let versions = api::fetch_all_versions().await?;

    let version = api::find_version(&versions, milestone);

    match version {
        Some(v) => {
            let source_name = match v.source {
                VersionSource::ChromeForTesting => "Chrome for Testing",
                VersionSource::ChromiumSnapshot => "Chromium Snapshot",
            };
            println!("  版本: {} ({})", v.version.white().bold(), source_name);

            download::download_and_install(&v.download_url, milestone).await?;

            println!(
                "\n  {} Chrome {} 安装完成!\n",
                "✓".green().bold(),
                milestone
            );
        }
        None => {
            println!(
                "  {} 未找到 milestone {} 的 Chrome 版本",
                "✗".red(),
                milestone
            );
            println!("\n  可用的 milestone:");

            // Show available milestones
            let mut milestones: Vec<_> = versions.iter().map(|v| v.milestone).collect();
            milestones.sort();
            for chunk in milestones.chunks(10) {
                let line: Vec<String> = chunk.iter().map(|m| format!("{}", m)).collect();
                println!("    {}", line.join(", "));
            }
            println!();
        }
    }

    Ok(())
}

fn cmd_uninstall(milestone: u32) -> Result<()> {
    println!(
        "{}",
        format!("\n🗑️  卸载 Chrome {}...\n", milestone).cyan().bold()
    );

    if !storage::is_installed(milestone)? {
        println!("  {} 版本 {} 未安装\n", "✗".yellow(), milestone);
        return Ok(());
    }

    storage::remove_version(milestone)?;

    println!(
        "  {} Chrome {} 已卸载\n",
        "✓".green().bold(),
        milestone
    );

    Ok(())
}

fn cmd_launch(milestone: u32, url: Option<String>) -> Result<()> {
    println!(
        "{}",
        format!("\n🚀 启动 Chrome {}...\n", milestone).cyan().bold()
    );

    if !storage::is_installed(milestone)? {
        println!(
            "  {} 版本 {} 未安装,请先安装: {}",
            "✗".red(),
            milestone,
            format!("chrome-tool install {}", milestone).green()
        );
        println!();
        return Ok(());
    }

    launcher::launch_chrome(milestone, url.as_deref())?;
    println!();

    Ok(())
}
