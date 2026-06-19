use crate::path;
use clap::Parser;
use std::path::{Path, PathBuf};
use std::process::Command;

#[derive(Parser)]
pub struct Args {
    /// Arguments passed straight through to m68k-elf-gdb (e.g. out/rom.out)
    #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
    args: Vec<String>,
}

/// Thin wrapper: locate the m68k gdb and exec it with the given args verbatim.
/// Typical use: debug a ROM against BlastEm's gdb remote stub
/// (`sgdkx gdb out/rom.out` then `target remote :1234` inside gdb).
pub fn run(args: &Args) {
    let exe = match find_gdb(&path::config_dir()) {
        Some(p) => p,
        None => {
            eprintln!("❌ m68k-elf-gdb not found. Run `sgdkx install` to download it.");
            std::process::exit(1);
        }
    };
    let status = Command::new(&exe)
        .args(&args.args)
        .status()
        .expect("Failed to run m68k-elf-gdb");
    std::process::exit(status.code().unwrap_or(1));
}

/// Locate the m68k gdb downloaded by `sgdkx install`:
/// `<config>/m68k-elf-gdb/bin/m68k-elf-gdb[.exe]` (a standalone download on every OS).
pub fn find_gdb(config_dir: &Path) -> Option<PathBuf> {
    let exe_name = if cfg!(target_os = "windows") {
        "m68k-elf-gdb.exe"
    } else {
        "m68k-elf-gdb"
    };
    let exe = config_dir.join("m68k-elf-gdb").join("bin").join(exe_name);
    exe.exists().then_some(exe)
}
