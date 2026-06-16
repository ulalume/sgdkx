use crate::path;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use toml_edit::DocumentMut;

pub fn run() {
    show_help_output();

    println!("\n🩺 sgdkx Environment Check");

    for tool in ["git", "make", "java", "compiledb"].iter() {
        check_tool(tool);
    }

    // doxygenはオプション
    check_tool("doxygen");

    #[cfg(not(target_os = "windows"))]
    check_tool("wine");

    let config_path = path::config_dir().join("config.toml");

    if config_path.exists() {
        let text = fs::read_to_string(&config_path).unwrap();
        let doc = text.parse::<DocumentMut>().unwrap();
        let sgdk_table = doc.get("sgdk").and_then(|v| v.as_inline_table());
        let path = sgdk_table
            .and_then(|tbl| tbl.get("path"))
            .and_then(|v| v.as_str())
            .unwrap_or("Unknown");
        let version = sgdk_table
            .and_then(|tbl| tbl.get("version"))
            .and_then(|v| v.as_str())
            .unwrap_or("Unknown");

        println!(
            "\n📝 sgdkx Configuration: {}",
            config_path.display()
        );
        println!("SGDK Path   : {}", path);
        println!("Version     : {}", version);

        let commit = Command::new("git")
            .args(["rev-parse", "HEAD"])
            .current_dir(path)
            .output()
            .ok()
            .and_then(|out| String::from_utf8(out.stdout).ok())
            .unwrap_or("Unknown".to_string());
        println!("Commit ID   : {}", commit.trim());

        // === Gens/Blastem Path Info 追加 ===
        let config_base = path::config_dir();

        // Gens
        let gens_path_config = doc
            .get("emulator")
            .and_then(|e| e.get("gens_path"))
            .and_then(|v| v.as_str());
        if let Some(path) = gens_path_config {
            println!("Gens Path   : {}", path);
        } else {
            let gens_path_opt = find_emulator_executable(&config_base, "gens");
            if let Some(gens_exe) = gens_path_opt {
                println!("Gens Path   : {}", gens_exe.display());
            } else {
                println!("Gens Path   : Not installed");
            }
        }

        // Blastem
        let blastem_path_config = doc
            .get("emulator")
            .and_then(|e| e.get("blastem_path"))
            .and_then(|v| v.as_str());
        if let Some(path) = blastem_path_config {
            println!("blastem Path: {}", path);
        } else {
            let blastem_path_opt = find_emulator_executable(&config_base, "blastem");
            if let Some(blastem_exe) = blastem_path_opt {
                println!(
                    "blastem Path: {}",
                    blastem_exe.display()
                );
            } else {
                println!("blastem Path: Not installed");
            }
        }

        // === SGDK Document Info ===
        let doc_index = Path::new(path).join("doc").join("html").join("index.html");
        if doc_index.exists() {
            println!(
                "\n📄 SGDK documentation: {}",
                doc_index
                    .canonicalize()
                    .expect("Failed to canonicalize path")
                    .to_str()
                    .unwrap()
                    .replace(r"\\?\", "")
            );
        } else {
            println!("⚠️  SGDK documentation not found.");
        }
    } else {
        println!("\n❌ config.toml not found. Please run `sgdkx setup`.");
    }
}

fn check_tool(tool: &str) {
    match which::which(tool) {
        Ok(path) => println!(
            "✅ {}: {}",
            tool,
            path.display()
        ),
        Err(_) => println!("❌ {}: not found", tool),
    }
}

fn find_emulator_executable(config_dir: &Path, emulator: &str) -> Option<PathBuf> {
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

fn show_help_output() {
    let exe = std::env::current_exe().unwrap_or_else(|_| "sgdkx".into());

    let status = Command::new(exe)
        .arg("help")
        .status()
        .expect("❌ Failed to execute sgdkx help");

    if !status.success() {
        eprintln!("⚠️  Failed to execute help command");
    }
}
