//! Auto-update checker: queries GitHub Releases API for newer versions.

const GITHUB_RELEASES_URL: &str = "https://api.github.com/repos/klivak/claudemeter/releases/latest";

const CURRENT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[derive(Debug, Clone)]
pub struct UpdateInfo {
    pub tag: String,
    pub html_url: String,
    /// Only needed to download the release; without `self-update` the user is
    /// sent to `html_url` instead.
    #[cfg(feature = "self-update")]
    pub asset_url: String,
    #[cfg(feature = "self-update")]
    pub checksum_url: String,
}

/// Check if a newer version is available on GitHub.
/// Returns release metadata if a newer version is available.
pub async fn check_for_update() -> Option<UpdateInfo> {
    let client = reqwest::Client::builder()
        .use_rustls_tls()
        .timeout(std::time::Duration::from_secs(10))
        .user_agent(format!("ClaudeMeter/{}", CURRENT_VERSION))
        .build()
        .ok()?;

    let resp = client.get(GITHUB_RELEASES_URL).send().await.ok()?;
    if !resp.status().is_success() {
        return None;
    }

    let json: serde_json::Value = resp.json().await.ok()?;
    let tag = json.get("tag_name")?.as_str()?;
    let html_url = json.get("html_url")?.as_str()?;
    // Asset URLs are only resolved when the build can actually install them —
    // otherwise a release whose assets are named differently would silently
    // suppress the notification.
    #[cfg(feature = "self-update")]
    let (asset_url, checksum_url) = {
        let assets = json.get("assets")?.as_array()?;
        let asset = assets.iter().find(|asset| {
            asset.get("name").and_then(|name| name.as_str()) == Some("claudemeter.exe")
        })?;
        let asset_url = asset.get("browser_download_url")?.as_str()?.to_string();
        let checksum_url = assets
            .iter()
            .find(|asset| {
                asset.get("name").and_then(|name| name.as_str()) == Some("claudemeter.exe.sha256")
            })
            .and_then(|asset| asset.get("browser_download_url"))
            .and_then(|url| url.as_str())
            .map(str::to_string)?;
        (asset_url, checksum_url)
    };

    let remote_ver = tag.trim_start_matches('v');
    if is_newer(remote_ver, CURRENT_VERSION) {
        Some(UpdateInfo {
            tag: tag.to_string(),
            html_url: html_url.to_string(),
            #[cfg(feature = "self-update")]
            asset_url,
            #[cfg(feature = "self-update")]
            checksum_url,
        })
    } else {
        None
    }
}

/// Original path of the running executable after a successful in-place swap.
/// Set by [`download_and_install`]; consumed by [`relaunch_updated`] at shutdown.
/// Captured *before* the swap because `current_exe()` follows the renamed file.
#[cfg(feature = "self-update")]
static UPDATED_EXE_PATH: std::sync::Mutex<Option<std::path::PathBuf>> = std::sync::Mutex::new(None);

/// Download an update, verify its checksum, and swap it into place natively.
///
/// Windows allows renaming a running executable, so no helper process is
/// needed: the live exe moves aside to `.exe.backup` and the verified download
/// takes its path. The app keeps running from the renamed file; the caller
/// quits its message loop, releases the single-instance mutex, and calls
/// [`relaunch_updated`]. (Deliberately no PowerShell here — a hidden
/// script replacing its parent exe is a classic malware pattern and was
/// tripping antivirus heuristics.)
///
/// Compiled out unless the `self-update` feature is on: even without PowerShell,
/// downloading an executable and overwriting yourself is what Defender's ML
/// models score highest. Shipped builds send the user to the release page.
#[cfg(feature = "self-update")]
pub async fn download_and_install(update: &UpdateInfo) -> Result<(), String> {
    let client = reqwest::Client::builder()
        .use_rustls_tls()
        .timeout(std::time::Duration::from_secs(120))
        .user_agent(format!("ClaudeMeter/{}", CURRENT_VERSION))
        .build()
        .map_err(|error| error.to_string())?;

    let current_exe = std::env::current_exe().map_err(|error| error.to_string())?;
    let download_path = current_exe.with_extension("exe.download");
    let bytes = client
        .get(&update.asset_url)
        .send()
        .await
        .map_err(|error| format!("Download failed: {error}"))?
        .error_for_status()
        .map_err(|error| format!("Download failed: {error}"))?
        .bytes()
        .await
        .map_err(|error| format!("Download failed: {error}"))?;

    let checksum = client
        .get(&update.checksum_url)
        .send()
        .await
        .map_err(|error| format!("Checksum download failed: {error}"))?
        .error_for_status()
        .map_err(|error| format!("Checksum download failed: {error}"))?
        .text()
        .await
        .map_err(|error| format!("Checksum download failed: {error}"))?;
    let expected_hash = parse_checksum(&checksum)?;

    let mut hasher = sha2::Sha256::new();
    use sha2::Digest;
    hasher.update(&bytes);
    let actual_hash = format!("{:x}", hasher.finalize());
    if expected_hash != actual_hash {
        return Err("Downloaded update failed SHA-256 verification".to_string());
    }

    std::fs::write(&download_path, &bytes).map_err(|error| error.to_string())?;
    swap_executable(&current_exe, &download_path)?;
    *UPDATED_EXE_PATH.lock().unwrap() = Some(current_exe);
    Ok(())
}

/// Move the running exe aside and put the verified download in its place.
/// Rolls the rename back if the second step fails, so the app keeps updating
/// and relaunching from its original path.
#[cfg(feature = "self-update")]
fn swap_executable(
    current_exe: &std::path::Path,
    download_path: &std::path::Path,
) -> Result<(), String> {
    let backup_path = current_exe.with_extension("exe.backup");
    // A stale backup from a previous update would make the rename fail.
    let _ = std::fs::remove_file(&backup_path);
    std::fs::rename(current_exe, &backup_path)
        .map_err(|error| format!("Could not move the running executable aside: {error}"))?;
    if let Err(error) = std::fs::rename(download_path, current_exe) {
        let _ = std::fs::rename(&backup_path, current_exe);
        return Err(format!("Could not move the update into place: {error}"));
    }
    Ok(())
}

/// True when an update has been swapped in and a relaunch is pending.
#[cfg(feature = "self-update")]
pub fn update_installed() -> bool {
    UPDATED_EXE_PATH.lock().unwrap().is_some()
}

/// Start the freshly installed executable. Call only after the single-instance
/// mutex has been released, right before process exit.
#[cfg(feature = "self-update")]
pub fn relaunch_updated() {
    let Some(exe) = UPDATED_EXE_PATH.lock().unwrap().take() else {
        return;
    };
    if let Err(error) = std::process::Command::new(&exe).spawn() {
        log::warn!("Failed to relaunch after update: {error}");
    }
}

/// Best-effort removal of leftovers from a previous update. The `.exe.backup`
/// of the old version stays locked until its process fully exits, so a failure
/// here is fine — the next launch will get it.
pub fn cleanup_stale_update_files() {
    if let Ok(current_exe) = std::env::current_exe() {
        let _ = std::fs::remove_file(current_exe.with_extension("exe.backup"));
        let _ = std::fs::remove_file(current_exe.with_extension("exe.download"));
    }
}

#[cfg(feature = "self-update")]
fn parse_checksum(content: &str) -> Result<String, String> {
    let value = content
        .split_whitespace()
        .next()
        .ok_or_else(|| "Checksum file was empty".to_string())?
        .to_lowercase();
    if value.len() != 64 || !value.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        return Err("Checksum file did not contain a valid SHA-256 hash".to_string());
    }
    Ok(value)
}

/// Simple semver comparison: returns true if `remote` > `current`.
fn is_newer(remote: &str, current: &str) -> bool {
    let parse =
        |s: &str| -> Vec<u32> { s.split('.').filter_map(|p| p.parse::<u32>().ok()).collect() };
    let r = parse(remote);
    let c = parse(current);
    for i in 0..3 {
        let rv = r.get(i).copied().unwrap_or(0);
        let cv = c.get(i).copied().unwrap_or(0);
        if rv > cv {
            return true;
        }
        if rv < cv {
            return false;
        }
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_newer() {
        assert!(is_newer("1.4.0", "1.3.6"));
        assert!(is_newer("2.0.0", "1.9.9"));
        assert!(!is_newer("1.3.6", "1.3.6"));
        assert!(!is_newer("1.3.5", "1.3.6"));
        assert!(is_newer("1.3.7", "1.3.6"));
    }

    #[test]
    fn test_is_newer_handles_missing_components() {
        assert!(is_newer("2", "1.9.9"));
        assert!(!is_newer("2.0", "2.0.0"));
        assert!(!is_newer("not-a-version", "2.0.0"));
    }

    #[test]
    #[cfg(feature = "self-update")]
    fn test_parse_checksum() {
        let hash = "A".repeat(64);
        assert_eq!(
            parse_checksum(&format!("{}  claudemeter.exe\n", hash)),
            Ok(hash.to_lowercase())
        );
        assert!(parse_checksum("").is_err());
        assert!(parse_checksum("not-a-sha256").is_err());
        assert!(parse_checksum(&"a".repeat(63)).is_err());
    }

    #[test]
    #[cfg(feature = "self-update")]
    fn test_swap_executable_replaces_and_backs_up() {
        let dir = std::env::temp_dir().join(format!("claudemeter-swap-ok-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let exe = dir.join("app.exe");
        let download = dir.join("app.exe.download");
        std::fs::write(&exe, b"old").unwrap();
        std::fs::write(&download, b"new").unwrap();

        swap_executable(&exe, &download).unwrap();

        assert_eq!(std::fs::read(&exe).unwrap(), b"new");
        assert_eq!(
            std::fs::read(dir.join("app.exe.backup")).unwrap(),
            b"old",
            "old exe must be preserved as backup"
        );
        assert!(!download.exists());
        std::fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    #[cfg(feature = "self-update")]
    fn test_swap_executable_rolls_back_when_download_missing() {
        let dir = std::env::temp_dir().join(format!("claudemeter-swap-rb-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let exe = dir.join("app.exe");
        let download = dir.join("app.exe.download"); // never created
        std::fs::write(&exe, b"old").unwrap();

        assert!(swap_executable(&exe, &download).is_err());

        assert_eq!(
            std::fs::read(&exe).unwrap(),
            b"old",
            "original exe must be restored on failure"
        );
        assert!(!dir.join("app.exe.backup").exists());
        std::fs::remove_dir_all(&dir).unwrap();
    }
}
