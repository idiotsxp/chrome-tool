use anyhow::{Context, Result};
use std::fs;
use std::path::PathBuf;

use crate::models::InstalledVersion;

/// Get the root directory for chrome-tool data: ~/.chrome-tool/
pub fn get_root_dir() -> Result<PathBuf> {
    let home = dirs::home_dir().context("无法获取用户主目录")?;
    Ok(home.join(".chrome-tool"))
}

/// Get the versions directory: ~/.chrome-tool/versions/
pub fn get_versions_dir() -> Result<PathBuf> {
    Ok(get_root_dir()?.join("versions"))
}

/// Get the cache directory: ~/.chrome-tool/cache/
pub fn get_cache_dir() -> Result<PathBuf> {
    Ok(get_root_dir()?.join("cache"))
}

/// Get the profiles directory: ~/.chrome-tool/profiles/
pub fn get_profiles_dir() -> Result<PathBuf> {
    Ok(get_root_dir()?.join("profiles"))
}

/// Get the installation directory for a specific milestone
pub fn get_version_dir(milestone: u32) -> Result<PathBuf> {
    Ok(get_versions_dir()?.join(milestone.to_string()))
}

/// Ensure all required directories exist
pub fn ensure_dirs() -> Result<()> {
    fs::create_dir_all(get_versions_dir()?)?;
    fs::create_dir_all(get_cache_dir()?)?;
    fs::create_dir_all(get_profiles_dir()?)?;
    Ok(())
}

/// Check if a milestone version is installed
pub fn is_installed(milestone: u32) -> Result<bool> {
    let dir = get_version_dir(milestone)?;
    Ok(dir.exists())
}

/// Find chrome.exe inside a version directory (searches recursively)
pub fn find_chrome_exe(milestone: u32) -> Result<Option<PathBuf>> {
    let version_dir = get_version_dir(milestone)?;
    if !version_dir.exists() {
        return Ok(None);
    }
    // Search for chrome.exe in the version directory
    find_exe_recursive(&version_dir, "chrome.exe")
}

fn find_exe_recursive(dir: &PathBuf, target: &str) -> Result<Option<PathBuf>> {
    if !dir.is_dir() {
        return Ok(None);
    }
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_file() {
            if let Some(name) = path.file_name() {
                if name.to_string_lossy().eq_ignore_ascii_case(target) {
                    return Ok(Some(path));
                }
            }
        } else if path.is_dir() {
            if let Some(found) = find_exe_recursive(&path, target)? {
                return Ok(Some(found));
            }
        }
    }
    Ok(None)
}

/// List all installed versions
pub fn list_installed() -> Result<Vec<InstalledVersion>> {
    let versions_dir = get_versions_dir()?;
    if !versions_dir.exists() {
        return Ok(vec![]);
    }

    let mut installed = Vec::new();
    for entry in fs::read_dir(&versions_dir)? {
        let entry = entry?;
        if entry.path().is_dir() {
            let name = entry.file_name().to_string_lossy().to_string();
            if let Ok(milestone) = name.parse::<u32>() {
                if let Ok(Some(exe)) = find_chrome_exe(milestone) {
                    installed.push(InstalledVersion {
                        milestone,
                        chrome_exe: exe,
                    });
                }
            }
        }
    }

    installed.sort_by_key(|v| v.milestone);
    Ok(installed)
}

/// Remove a version
pub fn remove_version(milestone: u32) -> Result<()> {
    let dir = get_version_dir(milestone)?;
    if dir.exists() {
        fs::remove_dir_all(&dir).context(format!("无法删除版本 {} 的目录", milestone))?;
    }
    // Also remove profile directory
    let profile_dir = get_profiles_dir()?.join(milestone.to_string());
    if profile_dir.exists() {
        fs::remove_dir_all(&profile_dir).ok(); // Ignore errors for profile cleanup
    }
    Ok(())
}
