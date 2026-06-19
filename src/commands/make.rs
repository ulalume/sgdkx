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
    let argv: Vec<&str> = args.args.iter().map(String::as_str).collect();
    let status = make_command(&doc, &argv)
        .status()
        .unwrap_or_else(|e| {
            eprintln!("❌ failed to run make: {e}");
            std::process::exit(1);
        });
    std::process::exit(status.code().unwrap_or(1));
}

/// Build a Command that runs `make <make_args>` with PATH prepared.
///
/// On Windows we run make *inside* SGDK's bundled MSYS `sh` (`sh -c "make ..."`).
/// Launched directly via CreateProcess, MSYS make's self-restart (after generating the
/// `.d` includes) and gcc `-flto`'s parallel make fail with quoted argv ('"make":
/// Command not found'). Running under MSYS sh gives the native environment SGDK expects
/// on Windows, where the restart/recursion work. On Unix we exec `make` directly.
pub fn make_command(doc: &DocumentMut, make_args: &[&str]) -> Command {
    prepend_tool_path(doc);
    #[cfg(target_os = "windows")]
    {
        let mut line = String::from("make");
        for a in make_args {
            line.push(' ');
            line.push_str(a);
        }
        let mut c = Command::new("sh");
        c.arg("-c").arg(line);
        return c;
    }
    #[cfg(not(target_os = "windows"))]
    {
        let mut c = Command::new("make");
        c.args(make_args);
        c
    }
}

pub fn load_config() -> DocumentMut {
    let config_path = path::config_dir().join("config.toml");
    if !config_path.exists() {
        eprintln!("❌ config.toml not found. Please run `sgdkx install` first.");
        std::process::exit(1);
    }
    fs::read_to_string(&config_path)
        .expect("config.toml read failed")
        .parse::<DocumentMut>()
        .expect("TOML parse failed")
}

/// Prepend the SGDK build-tool directories to THIS process's PATH (inherited by the
/// child make and its recipe commands). On Unix: bundled JRE, gcc toolchain, SGDK/bin.
/// On Windows: bundled JRE (for `java`) + SGDK/bin (bundled MSYS make.exe +
/// sh/rm/cp/mkdir/dlls + the m68k gcc.exe + native tools).
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
        // bundled JRE provides `java` for rescomp/sizebnd (common.mk uses bare `java`);
        // SGDK/bin provides make.exe + MSYS sh/rm/cp/mkdir + the bundled m68k gcc.exe +
        // native tools (the self-contained download bundle, like upstream's layout).
        if let Some(jre) = get_jre_path(doc) {
            prepend.push(Path::new(&jre).join("bin"));
        }
        prepend.push(sgdk_bin);
        let _ = get_toolchain_path(doc); // unused on Windows (toolchain lives in SGDK/bin)
    }

    let mut paths = prepend;
    if let Some(orig) = std::env::var_os("PATH") {
        paths.extend(std::env::split_paths(&orig));
    }
    let new_path = std::env::join_paths(&paths).expect("failed to build PATH");
    // Export GDK so the project's portable Makefile (`GDK ?= ...`) resolves to THIS install on
    // every platform. Forward-slashed because MSYS make on Windows expects `/` paths. (Older
    // Makefiles that hard-assign `GDK = <abs>` keep their value — a makefile `=` beats the env.)
    let gdk = sgdk_path.replace('\\', "/");
    // SAFETY: sgdkx is single-threaded here; set env right before spawning the build.
    unsafe {
        std::env::set_var("GDK", &gdk);
        std::env::set_var("PATH", &new_path);
    }
}
