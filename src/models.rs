use serde::Deserialize;
use std::collections::HashMap;

/// Source of the Chrome binary
#[derive(Debug, Clone, PartialEq)]
pub enum VersionSource {
    /// Chrome for Testing (v113+)
    ChromeForTesting,
    /// Chromium Browser Snapshots (v80-v112)
    ChromiumSnapshot,
}

/// Information about a Chrome version available for download/installed
#[derive(Debug, Clone)]
pub struct VersionInfo {
    pub milestone: u32,
    pub version: String,
    pub download_url: String,
    pub source: VersionSource,
}

/// Locally installed version
#[derive(Debug, Clone)]
pub struct InstalledVersion {
    pub milestone: u32,
    pub chrome_exe: std::path::PathBuf,
}

// ── Chrome for Testing API response models ──

#[derive(Debug, Deserialize)]
pub struct CftMilestoneResponse {
    pub milestones: HashMap<String, CftMilestone>,
}

#[derive(Debug, Deserialize)]
pub struct CftMilestone {
    pub milestone: String,
    pub version: String,
    pub downloads: CftDownloads,
}

#[derive(Debug, Deserialize)]
pub struct CftDownloads {
    pub chrome: Option<Vec<CftPlatformDownload>>,
}

#[derive(Debug, Deserialize)]
pub struct CftPlatformDownload {
    pub platform: String,
    pub url: String,
}
