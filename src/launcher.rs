use anyhow::{Context, Result};
use std::process::Command;

use crate::storage;

/// Launch a specific Chrome version with an isolated profile
pub fn launch_chrome(milestone: u32, url: Option<&str>) -> Result<()> {
    let chrome_exe = storage::find_chrome_exe(milestone)?
        .context(format!("版本 {} 未安装，请先运行: chrome-tool install {}", milestone, milestone))?;

    // Create isolated profile directory
    let profile_dir = storage::get_profiles_dir()?.join(milestone.to_string());
    std::fs::create_dir_all(&profile_dir)?;

    let mut cmd = Command::new(&chrome_exe);

    // Use isolated user-data-dir so different versions don't interfere
    cmd.arg(format!("--user-data-dir={}", profile_dir.display()));

    // Disable first-run experience for cleaner startup
    cmd.arg("--no-first-run");
    cmd.arg("--no-default-browser-check");

    // Open URL if specified
    if let Some(u) = url {
        cmd.arg(u);
    }

    println!("  启动 Chrome {} ...", milestone);
    println!("  路径: {}", chrome_exe.display());
    println!("  Profile: {}", profile_dir.display());

    cmd.spawn().context("无法启动 Chrome 进程")?;

    println!("  ✓ Chrome {} 已启动", milestone);

    Ok(())
}
