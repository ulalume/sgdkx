// Acquisition helpers: detect platform, download + extract release assets from the
// native-build repos (gcc toolchain, SGDK native bundle, BlastEm). Replaces the old
// Wine-based flow with native binaries downloaded from GitHub Releases.

use std::path::Path;
use std::process::Command;

// --- component sources (pinned) ---
// The toolchain is a separate download only on Unix; on Windows it's baked into the
// SGDK bundle, so these three are unused there (silence the Windows-only dead_code lint).
#[cfg_attr(target_os = "windows", allow(dead_code))]
pub const TOOLCHAIN_REPO: &str = "ulalume/m68k-toolchain-builds";
#[cfg_attr(target_os = "windows", allow(dead_code))]
pub const TOOLCHAIN_TAG: &str = "gcc13.2.0-1";
#[cfg_attr(target_os = "windows", allow(dead_code))]
pub const TOOLCHAIN_GCC_VERSION: &str = "13.2.0";
// m68k-elf-gdb (debugger) — a standalone per-platform download on every OS (incl. Windows;
// it is NOT a build tool, so it is not baked into the SGDK bundle like the gcc toolchain).
pub const GDB_REPO: &str = "ulalume/m68k-toolchain-builds";
pub const GDB_TAG: &str = "gdb16.2-1";
pub const GDB_VERSION: &str = "16.2";
pub const SGDK_NATIVE_REPO: &str = "ulalume/sgdk-native-builds";
/// Patched, debug-capable BlastEm fork (GDB remote + scriptable control socket + embedded
/// font). The default BlastEm source; builds all platforms from its own build.yml.
pub const BLASTEM_DEBUG_REPO: &str = "ulalume/blastem";
/// Upstream BlastEm, auto-tracked nightly (unmodified). The "original" alternative.
pub const BLASTEM_NIGHTLY_REPO: &str = "ulalume/blastem-builds";
pub const JRE_REPO: &str = "ulalume/jre-builds";
pub const JRE_TAG: &str = "jdk21-1";

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

/// GET a URL and parse JSON (used for the GitHub REST API), with retries.
pub fn http_json(url: &str) -> Result<serde_json::Value, String> {
    const ATTEMPTS: u32 = 3;
    let mut last_err = String::new();
    for attempt in 1..=ATTEMPTS {
        match try_json(url) {
            Ok(v) => return Ok(v),
            Err(e) => {
                last_err = e;
                if attempt < ATTEMPTS {
                    std::thread::sleep(std::time::Duration::from_secs(2));
                }
            }
        }
    }
    Err(format!("{last_err} (after {ATTEMPTS} attempts)"))
}

fn try_json(url: &str) -> Result<serde_json::Value, String> {
    let resp = http_client()
        .get(url)
        .send()
        .map_err(|e| format!("request failed: {e}"))?;
    if !resp.status().is_success() {
        return Err(format!("HTTP {} for {url}", resp.status()));
    }
    resp.json().map_err(|e| format!("invalid JSON from {url}: {e}"))
}

/// Download `url` to the file `dest`, streaming to disk with retries (large release
/// assets over flaky links otherwise fail with "error decoding response body").
pub fn download_to(url: &str, dest: &Path) -> Result<(), String> {
    const ATTEMPTS: u32 = 4;
    let mut last_err = String::new();
    for attempt in 1..=ATTEMPTS {
        match try_download(url, dest) {
            Ok(()) => return Ok(()),
            Err(e) => {
                last_err = e;
                if attempt < ATTEMPTS {
                    eprintln!("  download attempt {attempt}/{ATTEMPTS} failed ({last_err}); retrying...");
                    std::thread::sleep(std::time::Duration::from_secs(2 * attempt as u64));
                }
            }
        }
    }
    let _ = std::fs::remove_file(dest);
    Err(format!("{last_err} (after {ATTEMPTS} attempts)"))
}

fn try_download(url: &str, dest: &Path) -> Result<(), String> {
    let mut resp = http_client()
        .get(url)
        .send()
        .map_err(|e| format!("request failed: {e}"))?;
    if !resp.status().is_success() {
        return Err(format!("HTTP {} for {url}", resp.status()));
    }
    let mut file = std::fs::File::create(dest).map_err(|e| format!("create failed: {e}"))?;
    // stream the body to disk (low memory, fails fast on a dropped connection)
    std::io::copy(&mut resp, &mut file).map_err(|e| format!("read failed: {e}"))?;
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
        if let Some(name) = a["name"].as_str()
            && name.starts_with(name_prefix)
                && let Some(url) = a["browser_download_url"].as_str() {
                    return Ok(url.to_string());
                }
    }
    Err(format!(
        "no asset starting with '{name_prefix}' in {repo}@{tag}"
    ))
}

/// List release tag names for a repo, in GitHub's order (newest published first).
/// Used to populate the interactive version picker in `install`.
pub fn list_release_tags(repo: &str) -> Result<Vec<String>, String> {
    let json = http_json(&format!(
        "https://api.github.com/repos/{repo}/releases?per_page=100"
    ))?;
    let arr = json.as_array().ok_or("unexpected releases response")?;
    let tags: Vec<String> = arr
        .iter()
        .filter_map(|r| r["tag_name"].as_str().map(|s| s.to_string()))
        .collect();
    if tags.is_empty() {
        return Err(format!("no releases found in {repo}"));
    }
    Ok(tags)
}

/// List a repo's releases as `(tag, date)` pairs, newest first, for the version picker.
/// `date` is the upstream changeset date when the release body carries one (nightly builds
/// embed `Date: <date>`), otherwise the release's publish date — a "when is this" hint.
pub fn list_releases_with_dates(repo: &str) -> Result<Vec<(String, String)>, String> {
    let json = http_json(&format!(
        "https://api.github.com/repos/{repo}/releases?per_page=100"
    ))?;
    let arr = json.as_array().ok_or("unexpected releases response")?;
    let out: Vec<(String, String)> = arr
        .iter()
        .filter_map(|r| {
            let tag = r["tag_name"].as_str()?.to_string();
            let date = body_date(r["body"].as_str())
                .or_else(|| r["published_at"].as_str().map(short_date))
                .unwrap_or_default();
            Some((tag, date))
        })
        .collect();
    if out.is_empty() {
        return Err(format!("no releases found in {repo}"));
    }
    Ok(out)
}

/// The first `YYYY-MM-DD` appearing after a `Date:` marker in a release body (nightly builds
/// embed the upstream changeset date there). `None` if absent. Scans raw bytes (the target is
/// pure ASCII) so a multi-byte char elsewhere in the body can't cause a slice panic.
fn body_date(body: Option<&str>) -> Option<String> {
    let s = body?;
    let b = s.as_bytes();
    let start = s.find("Date:")?;
    (start..b.len().saturating_sub(9))
        .find(|&i| is_ymd(&b[i..i + 10]))
        .map(|i| String::from_utf8_lossy(&b[i..i + 10]).into_owned())
}

fn is_ymd(b: &[u8]) -> bool {
    b.len() == 10
        && b[4] == b'-'
        && b[7] == b'-'
        && b[..4].iter().all(u8::is_ascii_digit)
        && b[5..7].iter().all(u8::is_ascii_digit)
        && b[8..].iter().all(u8::is_ascii_digit)
}

/// `2026-06-22T01:23:45Z` -> `2026-06-22`.
fn short_date(iso: &str) -> String {
    iso.get(0..10).unwrap_or(iso).to_string()
}

/// Resolve the newest `master-<sha>` release tag from the SGDK native-builds repo.
pub fn latest_master_tag(repo: &str) -> Result<String, String> {
    let json = http_json(&format!(
        "https://api.github.com/repos/{repo}/releases?per_page=100"
    ))?;
    let arr = json.as_array().ok_or("unexpected releases response")?;
    // GitHub returns releases newest-first; take the first master-* tag.
    for r in arr {
        if let Some(tag) = r["tag_name"].as_str()
            && tag.starts_with("master-") {
                return Ok(tag.to_string());
            }
    }
    Err(format!("no master-* release found in {repo}"))
}
