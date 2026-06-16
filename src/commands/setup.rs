use crate::path;
use crate::release;
use clap::Parser;
use std::fs;
use std::path::Path;
use toml_edit::DocumentMut;

#[derive(Parser)]
pub struct Args {
    /// SGDK version: a release tag (e.g. v2.11) or "master" for the latest build
    #[arg(long, default_value = "master")]
    version: String,
}

pub fn run(args: &Args) {
    let config_dir = path::config_dir();
    fs::create_dir_all(&config_dir).expect("Failed to create config directory");

    #[cfg(not(target_os = "windows"))]
    setup_native(&config_dir, &args.version);

    #[cfg(target_os = "windows")]
    setup_windows(&config_dir, &args.version);
}

// --- Unix (macOS / Linux): native prebuilt toolchain + SGDK bundle ---
#[cfg(not(target_os = "windows"))]
fn setup_native(config_dir: &Path, version: &str) {
    let plat = release::platform();

    // 1. gcc 13 toolchain — download once, reuse across SGDK versions
    let toolchain_dir = config_dir.join("m68k-elf-toolchain");
    if toolchain_dir.join("bin").is_dir() {
        println!("✅ gcc toolchain already present: {}", toolchain_dir.display());
    } else {
        println!(
            "📥 Downloading gcc {} toolchain ({})...",
            release::TOOLCHAIN_GCC_VERSION, plat
        );
        let asset = format!(
            "m68k-elf-toolchain-gcc{}-{}.tar.gz",
            release::TOOLCHAIN_GCC_VERSION, plat
        );
        let url = release::asset_download_url(release::TOOLCHAIN_REPO, release::TOOLCHAIN_TAG, &asset);
        if let Err(e) = release::download_tar_gz(&url, config_dir) {
            eprintln!("❌ failed to fetch toolchain: {e}");
            std::process::exit(1);
        }
        println!("✅ gcc toolchain installed: {}", toolchain_dir.display());
    }

    // 2. resolve SGDK release tag
    let tag = if version == "master" {
        match release::latest_master_tag(release::SGDK_NATIVE_REPO) {
            Ok(t) => t,
            Err(e) => {
                eprintln!("❌ failed to resolve latest master release: {e}");
                std::process::exit(1);
            }
        }
    } else {
        version.to_string()
    };

    // 3. SGDK native bundle (native tools + libmd.a/libmd_debug.a + mac68k)
    let sgdk_dir = config_dir.join("SGDK");
    if sgdk_dir.exists() {
        println!("🗑️  Removing existing SGDK: {}", sgdk_dir.display());
        fs::remove_dir_all(&sgdk_dir).expect("Failed to remove existing SGDK directory");
    }
    println!("📥 Downloading SGDK {} ({})...", tag, plat);
    let asset = format!("sgdk-{}-{}.tar.gz", tag, plat);
    let url = release::asset_download_url(release::SGDK_NATIVE_REPO, &tag, &asset);
    if let Err(e) = release::download_tar_gz(&url, config_dir) {
        eprintln!("❌ failed to fetch SGDK {tag}: {e}");
        eprintln!("   (only release tags and 'master' are prebuilt; other commits are built on demand)");
        std::process::exit(1);
    }
    if !sgdk_dir.join("makefile.gen").exists() {
        eprintln!("❌ SGDK bundle missing makefile.gen — extraction problem?");
        std::process::exit(1);
    }

    write_config(config_dir, &sgdk_dir, &tag, Some(&toolchain_dir));
    println!("✅ SGDK setup complete: {}", sgdk_dir.display());
}

// --- Windows: clone upstream SGDK (ships its own bundled toolchain) ---
#[cfg(target_os = "windows")]
fn setup_windows(config_dir: &Path, version: &str) {
    use std::process::Command;
    use which::which;

    if which("git").is_err() {
        eprintln!("❌ 'git' not found. Please install it.");
        std::process::exit(1);
    }
    let target_dir = config_dir.join("SGDK");
    if target_dir.exists() {
        println!("🗑️  Removing existing SGDK: {}", target_dir.display());
        fs::remove_dir_all(&target_dir).expect("Failed to remove existing SGDK directory");
    }

    let is_commit_id = {
        let len = version.len();
        len >= 7 && len <= 40 && version.chars().all(|c| c.is_ascii_hexdigit())
    };

    println!("📥 Cloning SGDK from GitHub...");
    let status = if is_commit_id {
        Command::new("git")
            .args(["clone", "https://github.com/Stephane-D/SGDK", target_dir.to_str().unwrap()])
            .status()
            .expect("git clone failed")
    } else {
        Command::new("git")
            .args([
                "clone",
                "--branch",
                version,
                "https://github.com/Stephane-D/SGDK",
                target_dir.to_str().unwrap(),
            ])
            .status()
            .expect("git clone failed")
    };
    if !status.success() {
        eprintln!("❌ git clone failed");
        std::process::exit(1);
    }
    if is_commit_id {
        let co = Command::new("git")
            .args(["checkout", version])
            .current_dir(&target_dir)
            .status()
            .expect("git checkout failed");
        if !co.success() {
            eprintln!("❌ git checkout failed");
            std::process::exit(1);
        }
    }

    write_config(config_dir, &target_dir, version, None);
    println!("✅ SGDK setup complete: {}", target_dir.display());
}

fn write_config(config_dir: &Path, sgdk_dir: &Path, version: &str, toolchain_dir: Option<&Path>) {
    use toml_edit::{InlineTable, Item, Value};
    let config_path = config_dir.join("config.toml");
    let mut doc = if config_path.exists() {
        fs::read_to_string(&config_path)
            .expect("config.toml read failed")
            .parse::<DocumentMut>()
            .expect("TOML parse failed")
    } else {
        DocumentMut::new()
    };

    // stored as inline tables (read back via as_inline_table elsewhere)
    let mut sgdk = InlineTable::new();
    sgdk.insert("path", Value::from(canon(sgdk_dir)));
    sgdk.insert("version", Value::from(version));
    doc.insert("sgdk", Item::Value(Value::InlineTable(sgdk)));

    if let Some(tc) = toolchain_dir {
        let mut t = InlineTable::new();
        t.insert("path", Value::from(canon(tc)));
        doc.insert("toolchain", Item::Value(Value::InlineTable(t)));
    }
    fs::write(&config_path, doc.to_string()).expect("Failed to write config.toml");
}

/// Absolute path as a string, stripping the Windows `\\?\` verbatim prefix.
fn canon(p: &Path) -> String {
    p.canonicalize()
        .map(|c| c.to_string_lossy().replace(r"\\?\", ""))
        .unwrap_or_else(|_| p.to_string_lossy().to_string())
}
