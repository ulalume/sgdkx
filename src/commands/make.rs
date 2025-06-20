use dirs::config_dir;
use rust_i18n;
use std::fs;
use std::os::unix::fs::symlink;
use std::path::Path;
use std::process::Command;
use toml_edit::DocumentMut;

pub fn build_project(extra: &Vec<String>) {
    let dir = Path::new(".");
    if !dir.exists() {
        eprintln!("{}", rust_i18n::t!("project_dir_not_found"));
        std::process::exit(1);
    }

    let config_path = config_dir().unwrap().join("sgdktool/config.toml");
    let doc = fs::read_to_string(&config_path)
        .unwrap()
        .parse::<DocumentMut>()
        .unwrap();
    let (sgdk_path_str, _) = crate::commands::new::get_sgdk_config(&doc);
    let sgdk_path = Path::new(sgdk_path_str.unwrap_or_else(|| {
        eprintln!("SGDK path not found in config.toml.");
        std::process::exit(1);
    }));

    // If SGDK path contains spaces, create a temporary symlink
    let (effective_sgdk_path, temp_symlink) = if sgdk_path.to_str().unwrap().contains(' ') {
        println!("{}", rust_i18n::t!("compiledb_symlink_created"));
        let temp_dir = std::env::temp_dir();
        let symlink_path = temp_dir.join("sgdk_no_spaces");

        // Remove existing symlink if it exists
        if symlink_path.exists() {
            let _ = fs::remove_file(&symlink_path);
        }

        // Create symlink
        match symlink(sgdk_path, &symlink_path) {
            Ok(_) => (symlink_path, true),
            Err(_) => {
                eprintln!("{}", rust_i18n::t!("compiledb_symlink_failed"));
                std::process::exit(1);
            }
        }
    } else {
        (sgdk_path.to_path_buf(), false)
    };

    #[cfg(target_os = "windows")]
    let makefile = effective_sgdk_path.join("makefile.gen");
    #[cfg(not(target_os = "windows"))]
    let makefile = effective_sgdk_path.join("makefile_wine.gen");

    let sgdk_path_str = effective_sgdk_path.to_str().unwrap();

    let mut cmd = Command::new("make");
    cmd.current_dir(&dir)
        .arg(format!("GDK={}", sgdk_path_str))
        .arg("-f")
        .arg(&makefile);

    for arg in extra {
        cmd.arg(arg);
    }

    let status = cmd.status().expect("Failed to execute make");

    // Clean up temporary symlink
    if temp_symlink {
        let _ = fs::remove_file(&effective_sgdk_path);
    }

    std::process::exit(status.code().unwrap_or(1));
}
