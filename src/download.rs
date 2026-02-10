use anyhow::{Context, Result};
use futures_util::StreamExt;
use indicatif::{ProgressBar, ProgressStyle};
use std::fs::{self, File};
use std::io::{self, Write};
use std::path::Path;

use crate::storage;

/// Download a file from URL with a progress bar, returns the path to the downloaded file
pub async fn download_file(url: &str, milestone: u32) -> Result<std::path::PathBuf> {
    let cache_dir = storage::get_cache_dir()?;
    fs::create_dir_all(&cache_dir)?;

    let filename = format!("chrome-{}.zip", milestone);
    let dest = cache_dir.join(&filename);

    // If already cached, return
    if dest.exists() {
        println!("  使用缓存: {}", dest.display());
        return Ok(dest);
    }

    println!("  下载地址: {}", url);

    let client = reqwest::Client::new();
    let resp = client
        .get(url)
        .send()
        .await
        .context("下载请求失败")?;

    if !resp.status().is_success() {
        anyhow::bail!("下载失败，HTTP 状态码: {}", resp.status());
    }

    let total_size = resp.content_length().unwrap_or(0);

    let pb = ProgressBar::new(total_size);
    pb.set_style(
        ProgressStyle::default_bar()
            .template("  [{elapsed_precise}] [{bar:40.cyan/blue}] {bytes}/{total_bytes} ({eta})")
            .unwrap()
            .progress_chars("█▓░"),
    );

    let mut file = File::create(&dest).context("无法创建缓存文件")?;
    let mut stream = resp.bytes_stream();
    let mut downloaded: u64 = 0;

    while let Some(chunk) = stream.next().await {
        let chunk = chunk.context("下载数据块失败")?;
        file.write_all(&chunk)?;
        downloaded += chunk.len() as u64;
        pb.set_position(downloaded);
    }

    pb.finish_with_message("下载完成");
    println!();

    Ok(dest)
}

/// Extract a zip file to the target directory
pub fn extract_zip(zip_path: &Path, target_dir: &Path) -> Result<()> {
    println!("  正在解压...");

    fs::create_dir_all(target_dir)?;

    let file = File::open(zip_path).context("无法打开 ZIP 文件")?;
    let mut archive = zip::ZipArchive::new(file).context("无法解析 ZIP 文件")?;

    let total = archive.len();
    let pb = ProgressBar::new(total as u64);
    pb.set_style(
        ProgressStyle::default_bar()
            .template("  [{elapsed_precise}] [{bar:40.green/white}] {pos}/{len} 个文件")
            .unwrap()
            .progress_chars("█▓░"),
    );

    for i in 0..total {
        let mut entry = archive.by_index(i)?;
        let outpath = target_dir.join(
            entry
                .enclosed_name()
                .context("ZIP 中包含无效路径")?,
        );

        if entry.is_dir() {
            fs::create_dir_all(&outpath)?;
        } else {
            if let Some(parent) = outpath.parent() {
                fs::create_dir_all(parent)?;
            }
            let mut outfile = File::create(&outpath)?;
            io::copy(&mut entry, &mut outfile)?;
        }

        pb.set_position((i + 1) as u64);
    }

    pb.finish_with_message("解压完成");
    println!();

    Ok(())
}

/// Download and install a Chrome version
pub async fn download_and_install(url: &str, milestone: u32) -> Result<()> {
    storage::ensure_dirs()?;

    // Check if already installed
    if storage::is_installed(milestone)? {
        println!("  版本 {} 已安装，跳过", milestone);
        return Ok(());
    }

    // Download
    let zip_path = download_file(url, milestone).await?;

    // Extract
    let version_dir = storage::get_version_dir(milestone)?;
    extract_zip(&zip_path, &version_dir)?;

    // Verify chrome.exe exists
    match storage::find_chrome_exe(milestone)? {
        Some(exe) => {
            println!("  ✓ chrome.exe 位置: {}", exe.display());
        }
        None => {
            // Cleanup on failure
            storage::remove_version(milestone)?;
            anyhow::bail!("安装失败：未找到 chrome.exe");
        }
    }

    // Remove cached zip to save space
    if zip_path.exists() {
        fs::remove_file(&zip_path).ok();
    }

    Ok(())
}
