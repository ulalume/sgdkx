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
    setup(&config_dir, &args.version);
}

// All platforms use the same download model. The only OS difference: on Unix the gcc
// toolchain is a separately-cached component (reused across SGDK versions, put on PATH
// with the `m68k-elf-` prefix); on Windows the toolchain is baked into the self-contained
// SGDK bundle's `bin/` (common.mk's Windows branch expects every tool in `$(GDK)/bin`),
// so there is no separate toolchain download and no `[toolchain]` entry in config.toml.
fn setup(config_dir: &Path, version: &str) {
    let plat = release::platform();

    // 1. gcc 13 toolchain — Unix only (Windows bundles it inside the SGDK bundle's bin/).
    #[cfg(not(target_os = "windows"))]
    let toolchain_dir = {
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
            let url =
                release::asset_download_url(release::TOOLCHAIN_REPO, release::TOOLCHAIN_TAG, &asset);
            if let Err(e) = release::download_tar_gz(&url, config_dir) {
                eprintln!("❌ failed to fetch toolchain: {e}");
                std::process::exit(1);
            }
            println!("✅ gcc toolchain installed: {}", toolchain_dir.display());
        }
        toolchain_dir
    };

    // 1b. bundled minimal JRE (for rescomp/sizebnd) — all platforms; download once, reuse
    let jre_dir = config_dir.join("jre");
    if jre_dir.join("bin").is_dir() {
        println!("✅ bundled JRE already present: {}", jre_dir.display());
    } else {
        println!("📥 Downloading bundled JRE ({})...", plat);
        let asset = format!("jre-{}.tar.gz", plat);
        let url = release::asset_download_url(release::JRE_REPO, release::JRE_TAG, &asset);
        match release::download_tar_gz(&url, config_dir) {
            Ok(_) => println!("✅ JRE installed: {}", jre_dir.display()),
            Err(e) => println!("⚠️  bundled JRE unavailable ({e}); system Java will be used"),
        }
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

    // 4. prebuilt documentation (server-side doxygen), extracted into SGDK/doc/html
    println!("📥 Downloading SGDK documentation...");
    let docs_asset = format!("sgdk-docs-{}.tar.gz", tag);
    let docs_url = release::asset_download_url(release::SGDK_NATIVE_REPO, &tag, &docs_asset);
    match release::download_tar_gz(&docs_url, &sgdk_dir.join("doc")) {
        Ok(_) => println!("✅ documentation installed: {}", sgdk_dir.join("doc/html").display()),
        Err(e) => println!("⚠️  documentation not available ({e})"),
    }

    let jre_opt = if jre_dir.join("bin").is_dir() {
        Some(jre_dir.as_path())
    } else {
        None
    };
    // Unix: record the separate toolchain dir. Windows: toolchain lives in the SGDK
    // bundle's bin/, so no `[toolchain]` entry (get_toolchain_path -> None).
    #[cfg(not(target_os = "windows"))]
    write_config(config_dir, &sgdk_dir, &tag, Some(&toolchain_dir), jre_opt);
    #[cfg(target_os = "windows")]
    write_config(config_dir, &sgdk_dir, &tag, None, jre_opt);
    println!("✅ SGDK setup complete: {}", sgdk_dir.display());
}

fn write_config(
    config_dir: &Path,
    sgdk_dir: &Path,
    version: &str,
    toolchain_dir: Option<&Path>,
    jre_dir: Option<&Path>,
) {
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
    if let Some(jre) = jre_dir {
        let mut t = InlineTable::new();
        t.insert("path", Value::from(canon(jre)));
        doc.insert("jre", Item::Value(Value::InlineTable(t)));
    }
    fs::write(&config_path, doc.to_string()).expect("Failed to write config.toml");
}

/// Absolute path as a string, stripping the Windows `\\?\` verbatim prefix.
fn canon(p: &Path) -> String {
    p.canonicalize()
        .map(|c| c.to_string_lossy().replace(r"\\?\", ""))
        .unwrap_or_else(|_| p.to_string_lossy().to_string())
}
