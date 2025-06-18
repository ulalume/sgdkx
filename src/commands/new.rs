use std::path::Path;
use std::fs;
use dirs::config_dir;
use toml_edit::DocumentMut;
use rust_i18n;

pub fn create_project(name: &str) {
    let config_path = config_dir().unwrap().join("sgdktool/config.toml");

    // Check if config.toml exists
    if !config_path.exists() {
        eprintln!("{}", rust_i18n::t!("config_not_found_for_project"));
        std::process::exit(1);
    }

    let text = fs::read_to_string(&config_path).expect(&rust_i18n::t!("config_read_failed"));
    let doc = text
        .parse::<DocumentMut>()
        .expect(&rust_i18n::t!("toml_parse_failed"));
    let sgdk_path = Path::new(doc["sgdk"]["path"].as_str().unwrap());

    let template_path = sgdk_path.join("project").join("template");
    let dest_path = Path::new(name);

    if dest_path.exists() {
        eprintln!("{}", rust_i18n::t!("project_exists", name = name));
        std::process::exit(1);
    }

    println!("{}", rust_i18n::t!("creating_project", name = name));

    let mut opts = fs_extra::dir::CopyOptions::new();
    opts.copy_inside = true;
    fs_extra::dir::copy(&template_path, &dest_path, &opts).expect("Template copy failed");

    println!("{}", rust_i18n::t!("project_created", name = name));

    // Check for compiledb and run it if available
    println!("{}", rust_i18n::t!("compiledb_check"));
    if check_compiledb_available() {
        run_compiledb_make(&dest_path, &sgdk_path);
    }

    // Create .clangd configuration file
    create_clangd_config(&dest_path);

    // Create .vscode/c_cpp_properties.json
    create_vscode_config(&dest_path);

    // Create .gitignore
    create_gitignore(&dest_path);
}

pub fn check_compiledb_available() -> bool {
    match which::which("compiledb") {
        Ok(_) => {
            println!("{}", rust_i18n::t!("compiledb_found"));
            true
        }
        Err(_) => {
            println!("{}", rust_i18n::t!("compiledb_not_found"));
            false
        }
    }
}

pub fn run_compiledb_make(project_path: &Path, sgdk_path: &Path) -> bool {
    println!("{}", rust_i18n::t!("running_compiledb"));

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
        match std::os::unix::fs::symlink(sgdk_path, &symlink_path) {
            Ok(_) => (symlink_path, true),
            Err(_) => {
                println!("{}", rust_i18n::t!("compiledb_symlink_failed"));
                return false;
            }
        }
    } else {
        (sgdk_path.to_path_buf(), false)
    };

    let makefile = if cfg!(target_os = "windows") {
        effective_sgdk_path.join("makefile.gen")
    } else {
        effective_sgdk_path.join("makefile_wine.gen")
    };

    let sgdk_path_str = effective_sgdk_path.to_str().unwrap();

    let result = match std::process::Command::new("compiledb")
        .arg("make")
        .arg(format!("GDK={}", sgdk_path_str))
        .arg("-f")
        .arg(&makefile)
        .current_dir(project_path)
        .output()
    {
        Ok(output) => {
            if output.status.success() {
                println!("{}", rust_i18n::t!("compiledb_success"));
                true
            } else {
                println!("{}", rust_i18n::t!("compiledb_failed"));
                if !output.stderr.is_empty() {
                    eprintln!("Error: {}", String::from_utf8_lossy(&output.stderr));
                }
                if !output.stdout.is_empty() {
                    println!("Output: {}", String::from_utf8_lossy(&output.stdout));
                }
                false
            }
        }
        Err(e) => {
            println!("{}", rust_i18n::t!("compiledb_failed"));
            eprintln!("Error executing compiledb: {}", e);
            false
        }
    };

    // Post-process compile_commands.json to replace symlink paths with real paths
    if temp_symlink && result {
        fix_compile_commands_paths(project_path, &effective_sgdk_path, sgdk_path);
    }

    // Clean up temporary symlink
    if temp_symlink {
        let _ = fs::remove_file(&effective_sgdk_path);
    }

    result
}

pub fn fix_compile_commands_paths(project_path: &Path, symlink_path: &Path, real_sgdk_path: &Path) {
    let compile_commands_path = project_path.join("compile_commands.json");

    if let Ok(content) = fs::read_to_string(&compile_commands_path) {
        let symlink_str = symlink_path.to_str().unwrap();
        let real_str = real_sgdk_path.to_str().unwrap();

        let fixed_content = content.replace(symlink_str, real_str);

        if let Err(_) = fs::write(&compile_commands_path, fixed_content) {
            eprintln!("Warning: Failed to fix compile_commands.json paths");
        }
    }
}

pub fn create_clangd_config(project_path: &Path) {
    println!("{}", rust_i18n::t!("creating_clangd_config"));

    let clangd_content = r#"CompileFlags:
  Add:
    - '-DSGDK_GCC'
    - '-include'
    - 'types.h'
  Remove:
    - '-ffat-lto-objects'
    - '-externally_visible'
    - '-f*'
    - '-m68000'
Diagnostics:
  Suppress:
    - main_arg_wrong
"#;

    let clangd_path = project_path.join(".clangd");
    fs::write(clangd_path, clangd_content).expect("Failed to create .clangd file");
    println!("{}", rust_i18n::t!("clangd_config_created"));
}

pub fn create_vscode_config(project_path: &Path) {
    println!("{}", rust_i18n::t!("creating_vscode_config"));

    let vscode_dir = project_path.join(".vscode");
    if !vscode_dir.exists() {
        fs::create_dir_all(&vscode_dir).expect("Failed to create .vscode directory");
    }

    let cpp_properties_content = r#"{
    "configurations": [
      {
        "name": "sgdk",
        "cStandard": "c23",
        "intelliSenseMode": "gcc-x64",
        "compileCommands": "${workspaceFolder}/compile_commands.json"
      }
    ],
    "version": 4
}
"#;

    let cpp_properties_path = vscode_dir.join("c_cpp_properties.json");
    fs::write(cpp_properties_path, cpp_properties_content)
        .expect("Failed to create c_cpp_properties.json");
    println!("{}", rust_i18n::t!("vscode_config_created"));
}

pub fn create_gitignore(project_path: &Path) {
    println!("{}", rust_i18n::t!("creating_gitignore"));

    let gitignore_content = r#"/compile_commands.json
/.cache
/out
/res/**/*.h
"#;

    let gitignore_path = project_path.join(".gitignore");
    fs::write(gitignore_path, gitignore_content).expect("Failed to create .gitignore file");
    println!("{}", rust_i18n::t!("gitignore_created"));
} 