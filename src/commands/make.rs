use crate::commands::new::{get_jre_path, get_sgdk_config, get_toolchain_path};
use crate::path;
use clap::Parser;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use toml_edit::DocumentMut;

#[derive(Parser)]
pub struct Args {
    /// Arguments passed straight through to make (e.g. debug, clean, -j8)
    #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
    args: Vec<String>,
}

/// Thin wrapper around `make`: set up PATH so the build tools resolve, then exec make
/// with the given args verbatim. On Unix prepends the bundled JRE, gcc toolchain and
/// SGDK/bin; on Windows uses SGDK's bundled MSYS make.exe (+ sh/rm/cp/mkdir/dlls).
///
/// You can also run `make` directly if you put those directories on PATH yourself.
pub fn run(args: &Args) {
    let config_path = path::config_dir().join("config.toml");
    if !config_path.exists() {
        eprintln!("❌ config.toml not found. Please run `sgdkx setup` first.");
        std::process::exit(1);
    }
    let doc = fs::read_to_string(&config_path)
        .expect("config.toml read failed")
        .parse::<DocumentMut>()
        .expect("TOML parse failed");

    let sgdk_path = match get_sgdk_config(&doc).0 {
        Some(p) => p.to_string(),
        None => {
            eprintln!("❌ SGDK path not found in config.toml.");
            std::process::exit(1);
        }
    };
    let sgdk_bin = Path::new(&sgdk_path).join("bin");

    // directories to prepend to PATH for make + its recipes
    let mut prepend: Vec<PathBuf> = Vec::new();
    #[cfg(not(target_os = "windows"))]
    {
        if let Some(jre) = get_jre_path(&doc) {
            prepend.push(Path::new(&jre).join("bin"));
        }
        if let Some(tc) = get_toolchain_path(&doc) {
            prepend.push(Path::new(&tc).join("bin"));
        }
        prepend.push(sgdk_bin.clone());
    }
    #[cfg(target_os = "windows")]
    {
        // make.exe + sh/rm/cp/mkdir + msys dlls + bundled gcc all live here
        prepend.push(sgdk_bin.clone());
        let _ = (get_jre_path(&doc), get_toolchain_path(&doc)); // unused on Windows
    }

    let mut paths = prepend;
    if let Some(orig) = std::env::var_os("PATH") {
        paths.extend(std::env::split_paths(&orig));
    }
    let new_path = std::env::join_paths(&paths).expect("failed to build PATH");

    // resolve the make binary explicitly (avoids PATH-lookup ambiguity)
    let make_bin: PathBuf = {
        #[cfg(target_os = "windows")]
        {
            sgdk_bin.join("make.exe")
        }
        #[cfg(not(target_os = "windows"))]
        {
            which::which("make").unwrap_or_else(|_| {
                eprintln!("❌ 'make' not found. Please install make (build-essential / Xcode CLT).");
                std::process::exit(1);
            })
        }
    };

    let status = Command::new(&make_bin)
        .args(&args.args)
        .env("PATH", &new_path)
        .status()
        .unwrap_or_else(|e| {
            eprintln!("❌ failed to run make ({}): {e}", make_bin.display());
            std::process::exit(1);
        });
    std::process::exit(status.code().unwrap_or(1));
}
