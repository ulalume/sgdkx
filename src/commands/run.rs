use crate::path;
use clap::Parser;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

#[derive(Parser)]
pub struct Args {
    /// ROM file path (defaults to out/rom.bin)
    #[arg(long, default_value = "out/rom.bin")]
    rom: String,
}

pub fn run(args: &Args) {
    let config_dir = path::config_dir();
    let rom_path = &args.rom;

    if !Path::new(rom_path).exists() {
        eprintln!("ROM file not found: {}", rom_path);
        std::process::exit(1);
    }

    let exe = match find_blastem(&config_dir) {
        Some(p) => p,
        None => {
            eprintln!("BlastEm not found. Please run 'sgdkx setup-emu' first.");
            std::process::exit(1);
        }
    };

    let absolute_rom_path =
        fs::canonicalize(rom_path).expect("Failed to get absolute path for ROM file");

    println!("Running {} ...", exe.display());
    let status = Command::new(&exe)
        .arg(&absolute_rom_path)
        .status()
        .expect("Failed to run BlastEm");
    if !status.success() {
        eprintln!("BlastEm exited with error code: {:?}", status.code());
    }
}

/// Locate the native BlastEm executable under <config>/blastem, regardless of the
/// extracted layout (macOS: BlastEm.app/Contents/MacOS/blastem; Linux:
/// blastem-linux-*/blastem; Windows: blastem-win64-*/blastem.exe).
pub fn find_blastem(config_dir: &Path) -> Option<PathBuf> {
    let root = config_dir.join("blastem");
    if !root.exists() {
        return None;
    }
    let exe_name = if cfg!(target_os = "windows") {
        "blastem.exe"
    } else {
        "blastem"
    };
    let mut stack = vec![root];
    while let Some(dir) = stack.pop() {
        let entries = match fs::read_dir(&dir) {
            Ok(e) => e,
            Err(_) => continue,
        };
        for entry in entries.flatten() {
            let p = entry.path();
            if p.is_dir() {
                stack.push(p);
            } else if p.file_name().and_then(|n| n.to_str()) == Some(exe_name) {
                return Some(p);
            }
        }
    }
    None
}
