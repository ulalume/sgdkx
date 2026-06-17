use crate::path;
use std::fs;
use std::path::Path;
use std::process::Command;
use toml_edit::DocumentMut;

pub fn run() {
    show_help_output();

    println!("\n🩺 sgdkx Environment Check");

    // On Unix, `make` is the system one (required). On Windows, make is bundled with
    // SGDK (used via `sgdkx make`), and git is only needed for Windows setup (clone).
    #[cfg(not(target_os = "windows"))]
    check_tool("make");
    #[cfg(target_os = "windows")]
    check_tool("git");

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

        println!("\n📝 sgdkx Configuration: {}", config_path.display());
        println!("SGDK Path     : {}", path);
        println!("Version       : {}", version);

        let toolchain = doc
            .get("toolchain")
            .and_then(|v| v.as_inline_table())
            .and_then(|tbl| tbl.get("path"))
            .and_then(|v| v.as_str());
        match toolchain {
            Some(tc) => println!("Toolchain     : {}", tc),
            None => println!("Toolchain     : bundled (Windows)"),
        }

        // Bundled Java runtime (used by make for rescomp/sizebnd); falls back to
        // system Java only if absent.
        let jre = doc
            .get("jre")
            .and_then(|v| v.as_inline_table())
            .and_then(|tbl| tbl.get("path"))
            .and_then(|v| v.as_str());
        match jre {
            Some(j) => println!("JRE (bundled) : {}", j),
            None => match which::which("java") {
                Ok(p) => println!("JRE           : system java ({})", p.display()),
                Err(_) => println!("JRE           : ❌ none (no bundled JRE, no system java)"),
            },
        }

        // Commit ID is only meaningful on Windows (SGDK is a git clone there); the
        // Unix native bundle ships without .git — its commit is encoded in the
        // version tag (e.g. master-<sha>).
        #[cfg(target_os = "windows")]
        {
            let commit = Command::new("git")
                .args(["rev-parse", "HEAD"])
                .current_dir(path)
                .output()
                .ok()
                .and_then(|out| String::from_utf8(out.stdout).ok())
                .unwrap_or("Unknown".to_string());
            println!("Commit ID     : {}", commit.trim());
        }

        // BlastEm (the only supported emulator)
        let config_base = path::config_dir();
        let blastem = doc
            .get("emulator")
            .and_then(|e| e.get("blastem_path"))
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
            .or_else(|| {
                crate::commands::blastem::find_blastem(&config_base)
                    .map(|p| p.to_string_lossy().to_string())
            });
        match blastem {
            Some(p) => println!("BlastEm       : {}", p),
            None => println!("BlastEm       : Not installed"),
        }

        // SGDK documentation
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
        Ok(path) => println!("✅ {}: {}", tool, path.display()),
        Err(_) => println!("❌ {}: not found", tool),
    }
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
