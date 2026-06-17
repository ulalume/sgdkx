use crate::path;
use crate::release;
use clap::Parser;
use std::fs;
use toml_edit::{DocumentMut, InlineTable, Item, Value};

#[derive(Parser)]
pub struct Args {}

/// Download a native BlastEm build and record its path in config.toml.
pub fn run(_args: &Args) {
    let plat = release::platform();
    let config_dir = path::config_dir();
    let install_dir = config_dir.join("blastem");
    if install_dir.exists() {
        fs::remove_dir_all(&install_dir).expect("Failed to remove existing blastem directory");
    }
    fs::create_dir_all(&install_dir).expect("Failed to create install directory");

    // asset prefix + archive kind per platform (asset names are version-suffixed)
    let (prefix, is_zip) = match plat {
        "macos-arm64" => ("BlastEm-macOS-arm64-", true),
        "macos-x86_64" => ("BlastEm-macOS-x86_64-", true),
        "linux-x86_64" => ("blastem-linux-x86_64-", false),
        "linux-arm64" => ("blastem-linux-arm64-", false),
        "windows-x86_64" => ("blastem-win64-", true),
        other => {
            eprintln!("❌ no BlastEm build for platform {other}");
            std::process::exit(1);
        }
    };

    println!("📥 Downloading native BlastEm ({plat})...");
    let url = match release::find_asset_url(release::BLASTEM_REPO, "latest", prefix) {
        Ok(u) => u,
        Err(e) => {
            eprintln!("❌ failed to find BlastEm asset: {e}");
            std::process::exit(1);
        }
    };
    let res = if is_zip {
        release::download_zip(&url, &install_dir)
    } else {
        release::download_tar_gz(&url, &install_dir)
    };
    if let Err(e) = res {
        eprintln!("❌ failed to install BlastEm: {e}");
        std::process::exit(1);
    }

    match crate::commands::blastem::find_blastem(&config_dir) {
        Some(exe) => {
            let config_path = config_dir.join("config.toml");
            let mut doc = if config_path.exists() {
                fs::read_to_string(&config_path)
                    .unwrap()
                    .parse::<DocumentMut>()
                    .unwrap()
            } else {
                DocumentMut::new()
            };
            let mut emu = InlineTable::new();
            emu.insert("blastem_path", Value::from(exe.to_string_lossy().to_string()));
            doc.insert("emulator", Item::Value(Value::InlineTable(emu)));
            fs::write(&config_path, doc.to_string()).expect("Failed to write config.toml");
            println!("✅ BlastEm installed: {}", exe.display());
        }
        None => {
            eprintln!("❌ BlastEm binary not found after extraction in {}", install_dir.display());
            std::process::exit(1);
        }
    }
}
