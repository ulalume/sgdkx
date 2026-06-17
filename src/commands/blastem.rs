use crate::path;
use clap::Parser;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

#[derive(Parser)]
pub struct Args {
    /// Arguments passed straight through to BlastEm (e.g. out/rom.bin)
    #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
    args: Vec<String>,
}

/// Thin wrapper: locate the bundled BlastEm and exec it with the given args verbatim.
pub fn run(args: &Args) {
    let exe = match find_blastem(&path::config_dir()) {
        Some(p) => p,
        None => {
            eprintln!("BlastEm not found. Please run 'sgdkx setup-emu' first.");
            std::process::exit(1);
        }
    };
    let status = Command::new(&exe)
        .args(&args.args)
        .status()
        .expect("Failed to run BlastEm");
    std::process::exit(status.code().unwrap_or(1));
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
