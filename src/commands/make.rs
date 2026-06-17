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

/// Thin wrapper around `make`: prepend the build tool dirs to PATH, then run `make`
/// (bare) with the given args verbatim. You can also run `make` directly if you put
/// those directories on PATH yourself.
pub fn run(args: &Args) {
    let doc = load_config();
    let status = base_make_command(&doc)
        .args(&args.args)
        .status()
        .unwrap_or_else(|e| {
            eprintln!("❌ failed to run make: {e}");
            std::process::exit(1);
        });
    std::process::exit(status.code().unwrap_or(1));
}

/// A `make` Command with PATH prepared. On Windows, force `MAKE=make` (a command-line
/// variable assignment, highest precedence + exported to recipes) so make's restart
/// and gcc's `-flto` parallel link spawn a clean, PATH-resolvable `make` instead of a
/// quoted/absolute value the MSYS shell can't exec.
pub fn base_make_command(doc: &DocumentMut) -> Command {
    prepend_tool_path(doc);
    #[allow(unused_mut)]
    let mut c = Command::new("make");
    #[cfg(target_os = "windows")]
    c.arg("MAKE=make");
    c
}

pub fn load_config() -> DocumentMut {
    let config_path = path::config_dir().join("config.toml");
    if !config_path.exists() {
        eprintln!("❌ config.toml not found. Please run `sgdkx setup` first.");
        std::process::exit(1);
    }
    fs::read_to_string(&config_path)
        .expect("config.toml read failed")
        .parse::<DocumentMut>()
        .expect("TOML parse failed")
}

/// Prepend the SGDK build-tool directories to THIS process's PATH (inherited by the
/// child make and its recipe commands). On Unix: bundled JRE, gcc toolchain, SGDK/bin.
/// On Windows: SGDK/bin (bundled MSYS make.exe + sh/rm/cp/mkdir/dlls + gcc.exe).
///
/// We must modify the process PATH (not just the child's env) because on Windows the
/// executable lookup for a bare `make` uses the calling process's PATH. Running make
/// as a bare name (not an absolute path) keeps MSYS make's `$(MAKE)`/SHELL working.
pub fn prepend_tool_path(doc: &DocumentMut) {
    let sgdk_path = match get_sgdk_config(doc).0 {
        Some(p) => p.to_string(),
        None => {
            eprintln!("❌ SGDK path not found in config.toml.");
            std::process::exit(1);
        }
    };
    let sgdk_bin = Path::new(&sgdk_path).join("bin");

    let mut prepend: Vec<PathBuf> = Vec::new();
    #[cfg(not(target_os = "windows"))]
    {
        if let Some(jre) = get_jre_path(doc) {
            prepend.push(Path::new(&jre).join("bin"));
        }
        if let Some(tc) = get_toolchain_path(doc) {
            prepend.push(Path::new(&tc).join("bin"));
        }
        prepend.push(sgdk_bin);
    }
    #[cfg(target_os = "windows")]
    {
        prepend.push(sgdk_bin);
        let _ = (get_jre_path(doc), get_toolchain_path(doc)); // not used on Windows
    }

    let mut paths = prepend;
    if let Some(orig) = std::env::var_os("PATH") {
        paths.extend(std::env::split_paths(&orig));
    }
    let new_path = std::env::join_paths(&paths).expect("failed to build PATH");
    // SAFETY: sgdkx is single-threaded here; set PATH right before spawning the build.
    unsafe {
        std::env::set_var("PATH", &new_path);
    }
}
