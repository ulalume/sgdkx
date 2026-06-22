use crate::path;

pub fn run() {
    println!("\n🩺 sgdkx Environment Check");

    // On Unix, `make` is the system one (required). On Windows, make (and the whole
    // toolchain + MSYS shell) is bundled in SGDK/bin and used via `sgdkx make`, so no
    // system tool is required there.
    #[cfg(not(target_os = "windows"))]
    check_tool("make");

    if path::is_installed() {
        let sgdk_dir = path::sgdk_dir();
        let config_base = path::config_dir();

        println!("\n📝 sgdkx install: {}", config_base.display());
        println!("SGDK Path     : {}", sgdk_dir.display());
        println!(
            "Version       : {}",
            path::installed_version().unwrap_or_else(|| "Unknown".to_string())
        );

        match path::toolchain_dir() {
            Some(tc) => println!("Toolchain     : {}", tc.display()),
            None => println!("Toolchain     : bundled (Windows)"),
        }

        // Bundled Java runtime (used by make for rescomp/sizebnd); falls back to
        // system Java only if absent.
        match path::jre_dir() {
            Some(j) => println!("JRE (bundled) : {}", j.display()),
            None => match which::which("java") {
                Ok(p) => println!("JRE           : system java ({})", p.display()),
                Err(_) => println!("JRE           : ❌ none (no bundled JRE, no system java)"),
            },
        }

        // BlastEm (the only supported emulator) — located by search under <config>/blastem.
        match crate::commands::blastem::find_blastem(&config_base) {
            Some(p) => println!("BlastEm       : {}", p.display()),
            None => println!("BlastEm       : Not installed"),
        }

        // m68k-elf-gdb (Unix: downloaded by install; Windows: gdb.exe in the SGDK bundle)
        match crate::commands::gdb::find_gdb(&config_base) {
            Some(p) => println!("GDB           : {}", p.display()),
            None => println!("GDB           : Not installed"),
        }

        // SGDK documentation
        let doc_index = sgdk_dir.join("doc").join("html").join("index.html");
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
        println!("\n❌ Not installed. Please run `sgdkx install`.");
    }
}

#[cfg(not(target_os = "windows"))] // only the Unix branch checks a system tool (`make`)
fn check_tool(tool: &str) {
    match which::which(tool) {
        Ok(path) => println!("✅ {}: {}", tool, path.display()),
        Err(_) => println!("❌ {}: not found", tool),
    }
}
