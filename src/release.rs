// Acquisition helpers: detect platform, download + extract release assets from the
// native-build repos (gcc toolchain, SGDK native bundle, BlastEm). Replaces the old
// Wine-based flow with native binaries downloaded from GitHub Releases.

use std::path::Path;
use std::process::Command;

// --- component sources (pinned) ---
pub const TOOLCHAIN_REPO: &str = "ulalume/m68k-toolchain-builds";
pub const TOOLCHAIN_TAG: &str = "gcc13.2.0-1";
pub const TOOLCHAIN_GCC_VERSION: &str = "13.2.0";
pub const SGDK_NATIVE_REPO: &str = "ulalume/sgdk-native-builds";
pub const BLASTEM_REPO: &str = "ulalume/blastem-builds";

/// Platform slug used in release asset names.
/// linux-x86_64 / linux-arm64 / macos-arm64 / macos-x86_64 / windows-x86_64
pub fn platform() -> &'static str {
    match (std::env::consts::OS, std::env::consts::ARCH) {
        ("linux", "x86_64") => "linux-x86_64",
        ("linux", "aarch64") => "linux-arm64",
        ("macos", "aarch64") => "macos-arm64",
        ("macos", "x86_64") => "macos-x86_64",
        ("windows", "x86_64") => "windows-x86_64",
        (os, arch) => {
            eprintln!("❌ Unsupported platform: {os}/{arch}");
            std::process::exit(1);
        }
    }
}

fn http_client() -> reqwest::blocking::Client {
    reqwest::blocking::Client::builder()
        .user_agent("sgdkx")
        .build()
        .expect("failed to build HTTP client")
}

/// GET a URL and parse JSON (used for the GitHub REST API).
pub fn http_json(url: &str) -> Result<serde_json::Value, String> {
    let resp = http_client()
        .get(url)
        .send()
        .map_err(|e| format!("request failed: {e}"))?;
    if !resp.status().is_success() {
        return Err(format!("HTTP {} for {url}", resp.status()));
    }
    resp.json().map_err(|e| format!("invalid JSON from {url}: {e}"))
}

/// Download `url` to the file `dest`.
pub fn download_to(url: &str, dest: &Path) -> Result<(), String> {
    let resp = http_client()
        .get(url)
        .send()
        .map_err(|e| format!("download failed: {e}"))?;
    if !resp.status().is_success() {
        return Err(format!("HTTP {} for {url}", resp.status()));
    }
    let bytes = resp.bytes().map_err(|e| format!("read failed: {e}"))?;
    std::fs::write(dest, &bytes).map_err(|e| format!("write failed: {e}"))?;
    Ok(())
}

/// Extract a `.tar.gz` archive into `dest_dir` (uses the system `tar`, present on
/// Linux, macOS, and Windows 10+).
pub fn extract_tar_gz(archive: &Path, dest_dir: &Path) -> Result<(), String> {
    std::fs::create_dir_all(dest_dir).map_err(|e| e.to_string())?;
    let status = Command::new("tar")
        .arg("xzf")
        .arg(archive)
        .arg("-C")
        .arg(dest_dir)
        .status()
        .map_err(|e| format!("failed to run tar: {e}"))?;
    if !status.success() {
        return Err("tar extraction failed".into());
    }
    Ok(())
}

/// Download a `.tar.gz` from `url` and extract it into `dest_dir`.
pub fn download_tar_gz(url: &str, dest_dir: &Path) -> Result<(), String> {
    let tmp = tempfile::Builder::new()
        .suffix(".tar.gz")
        .tempfile()
        .map_err(|e| e.to_string())?;
    download_to(url, tmp.path())?;
    extract_tar_gz(tmp.path(), dest_dir)
}

/// Extract a `.zip` into `dest_dir`, preserving the executable bit on Unix.
pub fn extract_zip(archive: &Path, dest_dir: &Path) -> Result<(), String> {
    let file = std::fs::File::open(archive).map_err(|e| e.to_string())?;
    let mut zip = zip::ZipArchive::new(file).map_err(|e| format!("invalid zip: {e}"))?;
    std::fs::create_dir_all(dest_dir).map_err(|e| e.to_string())?;
    for i in 0..zip.len() {
        let mut entry = zip.by_index(i).map_err(|e| e.to_string())?;
        let out = dest_dir.join(entry.mangled_name());
        if entry.name().ends_with('/') {
            std::fs::create_dir_all(&out).map_err(|e| e.to_string())?;
            continue;
        }
        if let Some(parent) = out.parent() {
            std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        }
        let mut outfile = std::fs::File::create(&out).map_err(|e| e.to_string())?;
        std::io::copy(&mut entry, &mut outfile).map_err(|e| e.to_string())?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            if let Some(mode) = entry.unix_mode() {
                let _ = std::fs::set_permissions(&out, std::fs::Permissions::from_mode(mode));
            }
        }
    }
    Ok(())
}

/// Download a `.zip` from `url` and extract it into `dest_dir`.
pub fn download_zip(url: &str, dest_dir: &Path) -> Result<(), String> {
    let tmp = tempfile::Builder::new()
        .suffix(".zip")
        .tempfile()
        .map_err(|e| e.to_string())?;
    download_to(url, tmp.path())?;
    extract_zip(tmp.path(), dest_dir)
}

/// Direct download URL for a release asset whose name is known exactly.
pub fn asset_download_url(repo: &str, tag: &str, asset: &str) -> String {
    format!("https://github.com/{repo}/releases/download/{tag}/{asset}")
}

/// Find a release asset's download URL by name prefix (for versioned asset names).
/// `tag` may be a concrete tag or "latest".
pub fn find_asset_url(repo: &str, tag: &str, name_prefix: &str) -> Result<String, String> {
    let api = if tag == "latest" {
        format!("https://api.github.com/repos/{repo}/releases/latest")
    } else {
        format!("https://api.github.com/repos/{repo}/releases/tags/{tag}")
    };
    let json = http_json(&api)?;
    let assets = json["assets"]
        .as_array()
        .ok_or("release has no assets array")?;
    for a in assets {
        if let Some(name) = a["name"].as_str() {
            if name.starts_with(name_prefix) {
                if let Some(url) = a["browser_download_url"].as_str() {
                    return Ok(url.to_string());
                }
            }
        }
    }
    Err(format!(
        "no asset starting with '{name_prefix}' in {repo}@{tag}"
    ))
}

/// Resolve the newest `master-<sha>` release tag from the SGDK native-builds repo.
pub fn latest_master_tag(repo: &str) -> Result<String, String> {
    let json = http_json(&format!(
        "https://api.github.com/repos/{repo}/releases?per_page=100"
    ))?;
    let arr = json.as_array().ok_or("unexpected releases response")?;
    // GitHub returns releases newest-first; take the first master-* tag.
    for r in arr {
        if let Some(tag) = r["tag_name"].as_str() {
            if tag.starts_with("master-") {
                return Ok(tag.to_string());
            }
        }
    }
    Err(format!("no master-* release found in {repo}"))
}
