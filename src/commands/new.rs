use crate::path;
use clap::Parser;
use rust_i18n;
use std::fs;
use std::path::{Path, PathBuf};
use toml_edit::DocumentMut;

#[derive(Parser)]
pub struct Args {
    /// Project name (will be created as a directory)
    name: String,
}

/// シェル用にパスをエスケープする
pub fn escape_path(path: &str) -> String {
    // エスケープが必要な特殊文字のリスト
    const CHARS_TO_ESCAPE: &str = " \t\n;&|<>()$`\\\"'*?[]#~=";

    let mut result = String::with_capacity(path.len() * 2); // 余裕を持って確保

    for c in path.chars() {
        if CHARS_TO_ESCAPE.contains(c) {
            result.push('\\');
        }
        result.push(c);
    }

    result
}

pub fn run(args: &Args) {
    let name: &str = args.name.as_str();
    let config_path = path::config_dir().join("config.toml");

    // Check if config.toml exists
    if !config_path.exists() {
        eprintln!("{}", rust_i18n::t!("config_not_found_for_project"));
        std::process::exit(1);
    }

    let text = fs::read_to_string(&config_path).expect(&rust_i18n::t!("config_read_failed"));
    let doc = text
        .parse::<DocumentMut>()
        .expect(&rust_i18n::t!("toml_parse_failed"));
    let (sgdk_path_str, _) = get_sgdk_config(&doc);
    let sgdk_path = Path::new(sgdk_path_str.unwrap_or_else(|| {
        eprintln!("SGDK path not found in config.toml.");
        std::process::exit(1);
    }));

    let dest_path = Path::new(name);
    if dest_path.exists() {
        eprintln!("{}", rust_i18n::t!("project_exists", name = name));
        std::process::exit(1);
    }

    // テンプレート選択
    let template_path = select_template_dialoguer(sgdk_path);

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

    // Create platform-specific Makefile
    create_makefile(&dest_path, &sgdk_path);
}

/// sample配下をdialoguerで辿ってテンプレート選択。srcがあれば確定。デフォルトはsample/basics/hello-world。
fn select_template_dialoguer(sgdk_path: &Path) -> PathBuf {
    use dialoguer::{Select, theme::ColorfulTheme};
    let sample_root = sgdk_path.join("sample");

    // 再帰的にsrcディレクトリを持つテンプレート候補を収集
    fn find_templates_flat(base: &Path, rel: String, out: &mut Vec<(String, PathBuf)>) {
        if base.join("src").exists() {
            out.push((rel.clone(), base.to_path_buf()));
        }
        if let Ok(entries) = std::fs::read_dir(base) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    let name = entry.file_name().to_string_lossy().to_string();
                    let new_rel = if rel.is_empty() {
                        name
                    } else {
                        format!("{}/{}", rel, name)
                    };
                    find_templates_flat(&path, new_rel, out);
                }
            }
        }
    }

    let mut templates = Vec::new();
    find_templates_flat(&sample_root, String::new(), &mut templates);

    if templates.is_empty() {
        println!("No templates found in sample directory.");
        std::process::exit(1);
    }

    // アルファベット順（パス順）でソート
    let mut templates = templates;
    templates.sort_by(|a, b| a.0.cmp(&b.0));
    let items: Vec<_> = templates.iter().map(|(rel, _)| rel.clone()).collect();

    let selection = Select::with_theme(&ColorfulTheme::default())
        .with_prompt("Select a project template (Esc to cancel)")
        .items(&items)
        .default(0)
        .interact_opt()
        .unwrap();

    match selection {
        Some(idx) => {
            println!("Selected template: {}", templates[idx].0);
            templates[idx].1.clone()
        }
        None => {
            println!("Cancelled.");
            std::process::exit(0);
        }
    }
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

/// config.tomlのsgdkインラインテーブルからpath, versionを安全に取得
pub fn get_sgdk_config(doc: &DocumentMut) -> (Option<&str>, Option<&str>) {
    let sgdk_table = doc.get("sgdk").and_then(|v| v.as_inline_table());
    let path = sgdk_table
        .and_then(|tbl| tbl.get("path"))
        .and_then(|v| v.as_str());
    let version = sgdk_table
        .and_then(|tbl| tbl.get("version"))
        .and_then(|v| v.as_str());
    (path, version)
}

pub fn run_compiledb_make(project_path: &Path, sgdk_path: &Path) -> bool {
    println!("{}", rust_i18n::t!("running_compiledb"));

    let sgdk_path_str = sgdk_path.to_str().unwrap();
    let escaped_sgdk_path = escape_path(sgdk_path_str);
    println!("Using SGDK path: {}", escaped_sgdk_path);

    #[cfg(target_os = "windows")]
    let makefile = sgdk_path.join("makefile.gen");
    #[cfg(not(target_os = "windows"))]
    let makefile = sgdk_path.join("makefile_wine.gen");

    let result = match std::process::Command::new("compiledb")
        .arg("make")
        .arg(format!("GDK={}", escaped_sgdk_path))
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

    result
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
/res/**/*.rs
/Makefile
"#;

    let gitignore_path = project_path.join(".gitignore");
    fs::write(gitignore_path, gitignore_content).expect("Failed to create .gitignore file");
    println!("{}", rust_i18n::t!("gitignore_created"));
}

pub fn create_makefile(project_path: &Path, sgdk_path: &Path) {
    println!("{}", rust_i18n::t!("creating_makefile"));

    let sgdk_path_str = sgdk_path.to_string_lossy();
    let escaped_sgdk_path = escape_path(&sgdk_path_str);

    #[cfg(target_os = "windows")]
    let makefile_name = "makefile.gen";
    #[cfg(not(target_os = "windows"))]
    let makefile_name = "makefile_wine.gen";

    // Use standard environment paths on macOS and Linux when possible
    let makefile_content = format!(
        r#"# SGDK Makefile - Generated by sgdktool
# Note: This file is in .gitignore to avoid committing personal paths
# If needed, adjust the GDK path to match your SGDK installation

GDK = {}
include $(GDK)/{}
"#,
        escaped_sgdk_path, makefile_name,
    );

    let makefile_path = project_path.join("Makefile");
    fs::write(makefile_path, makefile_content).expect("Failed to create Makefile");
    println!("{}", rust_i18n::t!("makefile_created"));
}
