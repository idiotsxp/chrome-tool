use anyhow::{Context, Result};

use crate::models::*;

const CFT_MILESTONES_URL: &str =
    "https://googlechromelabs.github.io/chrome-for-testing/latest-versions-per-milestone-with-downloads.json";

/// Legacy Chromium versions (v80-v112) with hardcoded revision numbers.
/// Each entry: (milestone, full_version, chromium_revision)
const LEGACY_VERSIONS: &[(u32, &str, &str)] = &[
    (80,  "80.0.3987.163",  "722274"),
    (83,  "83.0.4103.116",  "756071"),
    (85,  "85.0.4183.121",  "818858"),
    (88,  "88.0.4324.150",  "827102"),
    (91,  "91.0.4472.124",  "870758"),
    (95,  "95.0.4638.69",   "929999"),
    (99,  "99.0.4844.84",   "972766"),
    (103, "103.0.5060.134", "1003031"),
    (106, "106.0.5249.119", "1036745"),
    (109, "109.0.5414.119", "1083080"),
    (112, "112.0.5615.121", "1108766"),
];

/// Get the list of legacy Chromium versions (v80-v112)
pub fn get_legacy_versions() -> Vec<VersionInfo> {
    LEGACY_VERSIONS
        .iter()
        .map(|(milestone, version, revision)| {
            let download_url = format!(
                "https://commondatastorage.googleapis.com/chromium-browser-snapshots/Win_x64/{}/chrome-win.zip",
                revision
            );
            VersionInfo {
                milestone: *milestone,
                version: version.to_string(),
                download_url,
                source: VersionSource::ChromiumSnapshot,
            }
        })
        .collect()
}

/// Fetch Chrome for Testing milestones (v113+) from the official API
pub async fn fetch_cft_milestones() -> Result<Vec<VersionInfo>> {
    let resp: CftMilestoneResponse = reqwest::get(CFT_MILESTONES_URL)
        .await
        .context("无法连接到 Chrome for Testing API")?
        .json()
        .await
        .context("无法解析 Chrome for Testing API 响应")?;

    let mut versions: Vec<VersionInfo> = Vec::new();

    for (_key, ms) in &resp.milestones {
        let milestone: u32 = ms.milestone.parse().unwrap_or(0);
        if milestone == 0 {
            continue;
        }

        // Find win64 download URL
        let url = ms
            .downloads
            .chrome
            .as_ref()
            .and_then(|platforms| {
                platforms
                    .iter()
                    .find(|p| p.platform == "win64")
                    .map(|p| p.url.clone())
            });

        if let Some(download_url) = url {
            versions.push(VersionInfo {
                milestone,
                version: ms.version.clone(),
                download_url,
                source: VersionSource::ChromeForTesting,
            });
        }
    }

    versions.sort_by_key(|v| v.milestone);
    Ok(versions)
}

/// Get all available versions (legacy + Chrome for Testing)
pub async fn fetch_all_versions() -> Result<Vec<VersionInfo>> {
    let mut all = get_legacy_versions();
    let cft = fetch_cft_milestones().await?;
    all.extend(cft);
    all.sort_by_key(|v| v.milestone);
    Ok(all)
}

/// Find a specific version by milestone from a list
pub fn find_version(versions: &[VersionInfo], milestone: u32) -> Option<&VersionInfo> {
    versions.iter().find(|v| v.milestone == milestone)
}
