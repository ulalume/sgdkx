use crate::release;
use crate::path;
use clap::Parser;
use std::fs;
use std::io::IsTerminal;
use std::path::Path;
use toml_edit::DocumentMut;

#[derive(Parser)]
pub struct Args {
    /// SGDK version to install: a release tag (e.g. v2.11), "master", or a master-<sha>.
    /// Omitted → interactive pick on a terminal, latest master when non-interactive.
    #[arg(short = 's', long = "sgdk")]
    sgdk: Option<String>,

    /// BlastEm version to install: a release tag (e.g. nightly-<sha>).
    /// Omitted → interactive pick on a terminal, latest when non-interactive.
    #[arg(short = 'b', long = "blastem")]
    blastem: Option<String>,
}

pub fn run(args: &Args) {
    let config_dir = path::config_dir();
    fs::create_dir_all(&config_dir).expect("Failed to create config directory");
    install(&config_dir, args);
}

// Idempotent install/reconfigure of the self-contained SGDK environment. Re-running is the
// supported way to *update* (it removes the old SGDK and re-downloads the requested version).
//
// OS difference: on Unix the gcc toolchain is a separately-cached component (reused across SGDK
// versions, put on PATH with the `m68k-elf-` prefix); on Windows the toolchain is baked into the
// self-contained SGDK bundle's `bin/`, so there is no separate toolchain download and no
// `[toolchain]` entry in config.toml.
fn install(config_dir: &Path, args: &Args) {
    let plat = release::platform();

    // Resolve versions up front (may prompt) so the rest of the flow is non-interactive.
    let sgdk_tag = resolve_sgdk_tag(args.sgdk.as_deref());
    let blastem_tag = resolve_blastem_tag(args.blastem.as_deref());

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

    // 1b. m68k-elf-gdb (debugger) — standalone download on every OS. Non-fatal.
    {
        let gdb_dir = config_dir.join("m68k-elf-gdb");
        if gdb_dir.join("bin").is_dir() {
            println!("✅ m68k-elf-gdb already present: {}", gdb_dir.display());
        } else {
            println!("📥 Downloading m68k-elf-gdb {} ({})...", release::GDB_VERSION, plat);
            let asset = format!("m68k-elf-gdb-{}-{}.tar.gz", release::GDB_VERSION, plat);
            let url = release::asset_download_url(release::GDB_REPO, release::GDB_TAG, &asset);
            match release::download_tar_gz(&url, config_dir) {
                Ok(_) => println!("✅ m68k-elf-gdb installed: {}", gdb_dir.display()),
                Err(e) => println!("⚠️  m68k-elf-gdb unavailable ({e}); `sgdkx gdb` will not work"),
            }
        }
    }

    // 1c. bundled minimal JRE (for rescomp/sizebnd) — all platforms; download once, reuse
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

    // 2. SGDK native bundle (native tools + libmd.a/libmd_debug.a + mac68k)
    let sgdk_dir = config_dir.join("SGDK");
    if sgdk_dir.exists() {
        println!("🗑️  Removing existing SGDK: {}", sgdk_dir.display());
        fs::remove_dir_all(&sgdk_dir).expect("Failed to remove existing SGDK directory");
    }
    println!("📥 Downloading SGDK {} ({})...", sgdk_tag, plat);
    let asset = format!("sgdk-{}-{}.tar.gz", sgdk_tag, plat);
    let url = release::asset_download_url(release::SGDK_NATIVE_REPO, &sgdk_tag, &asset);
    if let Err(e) = release::download_tar_gz(&url, config_dir) {
        eprintln!("❌ failed to fetch SGDK {sgdk_tag}: {e}");
        eprintln!("   (only release tags and 'master' are prebuilt; other commits are built on demand)");
        std::process::exit(1);
    }
    if !sgdk_dir.join("makefile.gen").exists() {
        eprintln!("❌ SGDK bundle missing makefile.gen — extraction problem?");
        std::process::exit(1);
    }

    // 3. prebuilt documentation (server-side doxygen), extracted into SGDK/doc/html
    println!("📥 Downloading SGDK documentation...");
    let docs_asset = format!("sgdk-docs-{}.tar.gz", sgdk_tag);
    let docs_url = release::asset_download_url(release::SGDK_NATIVE_REPO, &sgdk_tag, &docs_asset);
    match release::download_tar_gz(&docs_url, &sgdk_dir.join("doc")) {
        Ok(_) => println!("✅ documentation installed: {}", sgdk_dir.join("doc/html").display()),
        Err(e) => println!("⚠️  documentation not available ({e})"),
    }

    // 4. native BlastEm emulator — standalone download. Non-fatal (only disables `sgdkx blastem`).
    let blastem_exe = download_blastem(config_dir, &blastem_tag);

    let jre_opt = jre_dir.join("bin").is_dir().then_some(jre_dir.as_path());
    // Unix: record the separate toolchain dir. Windows: toolchain lives in the SGDK
    // bundle's bin/, so no `[toolchain]` entry (get_toolchain_path -> None).
    #[cfg(not(target_os = "windows"))]
    write_config(config_dir, &sgdk_dir, &sgdk_tag, Some(&toolchain_dir), jre_opt, blastem_exe.as_deref());
    #[cfg(target_os = "windows")]
    write_config(config_dir, &sgdk_dir, &sgdk_tag, None, jre_opt, blastem_exe.as_deref());
    println!("✅ SGDK install complete: {}", sgdk_dir.display());
}

/// Resolve the SGDK release tag to install.
/// Explicit flag wins ("master" → newest master-<sha>); otherwise interactive on a terminal,
/// latest master when non-interactive (scriptable default).
fn resolve_sgdk_tag(explicit: Option<&str>) -> String {
    if let Some(v) = explicit {
        return if v == "master" { latest_master_or_exit() } else { v.to_string() };
    }
    if std::io::stdin().is_terminal() {
        match release::list_release_tags(release::SGDK_NATIVE_REPO) {
            Ok(tags) => pick("Select an SGDK version", &tags),
            Err(e) => {
                eprintln!("⚠️  could not list SGDK versions ({e}); using latest master");
                latest_master_or_exit()
            }
        }
    } else {
        latest_master_or_exit()
    }
}

/// Resolve the BlastEm release tag. Explicit flag wins; otherwise interactive on a terminal,
/// "latest" when non-interactive.
fn resolve_blastem_tag(explicit: Option<&str>) -> String {
    if let Some(v) = explicit {
        return v.to_string();
    }
    if std::io::stdin().is_terminal() {
        match release::list_release_tags(release::BLASTEM_REPO) {
            Ok(tags) => pick("Select a BlastEm version", &tags),
            Err(e) => {
                eprintln!("⚠️  could not list BlastEm versions ({e}); using latest");
                "latest".to_string()
            }
        }
    } else {
        "latest".to_string()
    }
}

fn latest_master_or_exit() -> String {
    match release::latest_master_tag(release::SGDK_NATIVE_REPO) {
        Ok(t) => t,
        Err(e) => {
            eprintln!("❌ failed to resolve latest master release: {e}");
            std::process::exit(1);
        }
    }
}

/// Interactive single-select over `items` (default = first/newest). Esc cancels the install.
fn pick(prompt: &str, items: &[String]) -> String {
    use dialoguer::{Select, theme::ColorfulTheme};
    let selection = Select::with_theme(&ColorfulTheme::default())
        .with_prompt(format!("{prompt} (Esc to cancel)"))
        .items(items)
        .default(0)
        .interact_opt()
        .unwrap();
    match selection {
        Some(idx) => items[idx].clone(),
        None => {
            println!("Cancelled.");
            std::process::exit(0);
        }
    }
}

/// Download a native BlastEm build into `<config>/blastem` and return its executable path.
/// Returns None (after a warning) on any failure — BlastEm is optional.
fn download_blastem(config_dir: &Path, tag: &str) -> Option<std::path::PathBuf> {
    let plat = release::platform();
    let install_dir = config_dir.join("blastem");
    if install_dir.exists() {
        let _ = fs::remove_dir_all(&install_dir);
    }
    if let Err(e) = fs::create_dir_all(&install_dir) {
        eprintln!("⚠️  could not create blastem dir ({e}); `sgdkx blastem` will not work");
        return None;
    }

    // asset prefix + archive kind per platform (asset names are version-suffixed)
    let (prefix, is_zip) = match plat {
        "macos-arm64" => ("BlastEm-macOS-arm64-", true),
        "macos-x86_64" => ("BlastEm-macOS-x86_64-", true),
        "linux-x86_64" => ("blastem-linux-x86_64-", false),
        "linux-arm64" => ("blastem-linux-arm64-", false),
        "windows-x86_64" => ("blastem-win64-", true),
        other => {
            eprintln!("⚠️  no BlastEm build for platform {other}; `sgdkx blastem` will not work");
            return None;
        }
    };

    println!("📥 Downloading native BlastEm {tag} ({plat})...");
    let url = match release::find_asset_url(release::BLASTEM_REPO, tag, prefix) {
        Ok(u) => u,
        Err(e) => {
            eprintln!("⚠️  BlastEm unavailable ({e}); `sgdkx blastem` will not work");
            return None;
        }
    };
    let res = if is_zip {
        release::download_zip(&url, &install_dir)
    } else {
        release::download_tar_gz(&url, &install_dir)
    };
    if let Err(e) = res {
        eprintln!("⚠️  failed to install BlastEm ({e}); `sgdkx blastem` will not work");
        return None;
    }
    match crate::commands::blastem::find_blastem(config_dir) {
        Some(exe) => {
            println!("✅ BlastEm installed: {}", exe.display());
            Some(exe)
        }
        None => {
            eprintln!("⚠️  BlastEm binary not found after extraction; `sgdkx blastem` will not work");
            None
        }
    }
}

fn write_config(
    config_dir: &Path,
    sgdk_dir: &Path,
    version: &str,
    toolchain_dir: Option<&Path>,
    jre_dir: Option<&Path>,
    blastem_exe: Option<&Path>,
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
    if let Some(exe) = blastem_exe {
        let mut t = InlineTable::new();
        t.insert("blastem_path", Value::from(exe.to_string_lossy().to_string()));
        doc.insert("emulator", Item::Value(Value::InlineTable(t)));
    }
    fs::write(&config_path, doc.to_string()).expect("Failed to write config.toml");
}

/// Absolute path as a string, stripping the Windows `\\?\` verbatim prefix.
fn canon(p: &Path) -> String {
    p.canonicalize()
        .map(|c| c.to_string_lossy().replace(r"\\?\", ""))
        .unwrap_or_else(|_| p.to_string_lossy().to_string())
}
