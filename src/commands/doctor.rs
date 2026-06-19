use crate::commands::new::get_sgdk_config;
use crate::path;
use std::fs;
use std::path::Path;
use toml_edit::DocumentMut;

pub fn run() {
    println!("\n🩺 sgdkx Environment Check");

    // On Unix, `make` is the system one (required). On Windows, make (and the whole
    // toolchain + MSYS shell) is bundled in SGDK/bin and used via `sgdkx make`, so no
    // system tool is required there.
    #[cfg(not(target_os = "windows"))]
    check_tool("make");

    let config_path = path::config_dir().join("config.toml");

    if config_path.exists() {
        let text = fs::read_to_string(&config_path).unwrap();
        let doc = text.parse::<DocumentMut>().unwrap();
        let (path_opt, version_opt) = get_sgdk_config(&doc);
        let path = path_opt.unwrap_or("Unknown");
        let version = version_opt.unwrap_or("Unknown");

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

        // No commit-id check: on all platforms SGDK is now a versioned prebuilt bundle
        // shipped without .git — its commit is encoded in the version tag above
        // (e.g. master-<sha>).

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

        // m68k-elf-gdb (Unix: downloaded by setup; Windows: gdb.exe in the SGDK bundle)
        match crate::commands::gdb::find_gdb(&config_base) {
            Some(p) => println!("GDB           : {}", p.display()),
            None => println!("GDB           : Not installed"),
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
        println!("\n❌ config.toml not found. Please run `sgdkx install`.");
    }
}

#[cfg(not(target_os = "windows"))] // only the Unix branch checks a system tool (`make`)
fn check_tool(tool: &str) {
    match which::which(tool) {
        Ok(path) => println!("✅ {}: {}", tool, path.display()),
        Err(_) => println!("❌ {}: not found", tool),
    }
}
