use crate::path;
use clap::Parser;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
#[cfg(not(target_os = "windows"))]
use which::which;

#[derive(Parser)]
pub struct Args {
    /// Emulator to use (gens or blastem, defaults to available emulator)
    #[arg(long)]
    emulator: Option<String>,

    /// ROM file path (defaults to out/rom.bin)
    #[arg(long, default_value = "out/rom.bin")]
    rom: String,
}

pub fn run(args: &Args) {
    let config_dir = path::config_dir();

    let rom_path = &args.rom;
    let emulator = &args.emulator;

    // Check if ROM file exists
    if !Path::new(rom_path).exists() {
        eprintln!("ROM file not found: {}", rom_path);
        std::process::exit(1);
    }

    let emulator_to_use = if let Some(emu) = emulator {
        emu.to_string()
    } else {
        // Auto-detect available emulator
        if find_emulator_executable(&config_dir, "gens").is_some() {
            "gens".to_string()
        } else if find_emulator_executable(&config_dir, "blastem").is_some() {
            "blastem".to_string()
        } else {
            eprintln!("No emulator found. Please run 'sgdkx setup-emu' first.");
            std::process::exit(1);
        }
    };

    let emulator_path = find_emulator_executable(&config_dir, &emulator_to_use);

    if let Some(exe_path) = emulator_path {
        #[cfg(target_os = "windows")]
        {
            run_direct(&exe_path, rom_path);
        }
        #[cfg(not(target_os = "windows"))]
        {
            run_with_wine(&exe_path, rom_path);
        }
    } else {
        eprintln!(
            "Emulator '{}' not found. Please run 'sgdkx setup-emu {}' first.",
            emulator_to_use, emulator_to_use
        );
        std::process::exit(1);
    }
}

pub fn find_emulator_executable(config_dir: &Path, emulator: &str) -> Option<PathBuf> {
    let emulator_dir = config_dir.join(emulator);

    match emulator {
        "gens" => {
            // Look for gens.exe in various possible locations
            let possible_paths = vec![
                emulator_dir.join("gens.exe"),
                emulator_dir.join("Gens_KMod_v0.7.3").join("gens.exe"),
            ];

            for path in possible_paths {
                if path.exists() {
                    return Some(path);
                }
            }
        }
        "blastem" => {
            // Look for blastem.exe in various possible locations
            let possible_paths = vec![emulator_dir.join("blastem.exe")];

            // Also look for blastem-win64-* directories
            if let Ok(entries) = fs::read_dir(&emulator_dir) {
                for entry in entries {
                    if let Ok(entry) = entry {
                        let path = entry.path();
                        if path.is_dir()
                            && path
                                .file_name()
                                .unwrap()
                                .to_str()
                                .unwrap()
                                .starts_with("blastem-win64")
                        {
                            let exe_path = path.join("blastem.exe");
                            if exe_path.exists() {
                                return Some(exe_path);
                            }
                        }
                    }
                }
            }

            for path in possible_paths {
                if path.exists() {
                    return Some(path);
                }
            }
        }
        _ => {}
    }

    None
}

#[cfg(not(target_os = "windows"))]
pub fn run_with_wine(exe_path: &Path, rom_path: &str) {
    // Check if wine is available
    if which("wine").is_err() {
        eprintln!(
            "Wine is not installed or not in PATH. Please install wine to run Windows emulators."
        );
        std::process::exit(1);
    }

    println!("Running {} with wine...", exe_path.display());

    let absolute_rom_path =
        fs::canonicalize(rom_path).expect("Failed to get absolute path for ROM file");

    let status = Command::new("wine")
        .arg(exe_path)
        .arg(&absolute_rom_path)
        .status()
        .expect("Failed to run emulator with wine");

    if !status.success() {
        eprintln!("Emulator exited with error code: {:?}", status.code());
    }
}

#[cfg(target_os = "windows")]
pub fn run_direct(exe_path: &Path, rom_path: &str) {
    println!("Running {} ...", exe_path.display());

    let absolute_rom_path =
        fs::canonicalize(rom_path).expect("Failed to get absolute path for ROM file");

    let status = Command::new(exe_path)
        .arg(&absolute_rom_path)
        .status()
        .expect("Failed to run emulator");

    if !status.success() {
        eprintln!("Emulator exited with error code: {:?}", status.code());
    }
}
