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
            eprintln!("❌ m68k-elf-gdb not found. Run `sgdkx setup` to download it.");
            std::process::exit(1);
        }
    };
    let status = Command::new(&exe)
        .args(&args.args)
        .status()
        .expect("Failed to run m68k-elf-gdb");
    std::process::exit(status.code().unwrap_or(1));
}

/// Locate the m68k gdb.
/// Unix: `<config>/m68k-elf-gdb/bin/m68k-elf-gdb` (downloaded by `sgdkx setup`).
/// Windows: `gdb.exe` in the SGDK bundle's `bin/` (shipped inside the bundle, like the rest
/// of the Windows toolchain).
pub fn find_gdb(config_dir: &Path) -> Option<PathBuf> {
    #[cfg(not(target_os = "windows"))]
    {
        let exe = config_dir
            .join("m68k-elf-gdb")
            .join("bin")
            .join("m68k-elf-gdb");
        exe.exists().then_some(exe)
    }
    #[cfg(target_os = "windows")]
    {
        let _ = config_dir;
        // gdb.exe lives in the SGDK bundle's bin/ — read its path from config.toml.
        use toml_edit::DocumentMut;
        let doc = std::fs::read_to_string(path::config_dir().join("config.toml"))
            .ok()?
            .parse::<DocumentMut>()
            .ok()?;
        let sgdk = doc
            .get("sgdk")
            .and_then(|v| v.as_inline_table())
            .and_then(|t| t.get("path"))
            .and_then(|v| v.as_str())?;
        let exe = Path::new(sgdk).join("bin").join("gdb.exe");
        exe.exists().then_some(exe)
    }
}
