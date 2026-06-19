use crate::path;
use clap::Parser;
use std::fs;
use std::io::IsTerminal;
use std::path::{Path, PathBuf};
use toml_edit::DocumentMut;

#[derive(Parser)]
pub struct Args {
    /// Project name (will be created as a directory)
    name: String,
    /// Template to use, by its path under SGDK/sample (e.g. basics/hello-world).
    /// Omitted → interactive pick on a terminal; required when non-interactive.
    #[arg(short = 't', long = "template")]
    template: Option<String>,
}

pub fn run(args: &Args) {
    let name: &str = args.name.as_str();
    let config_path = path::config_dir().join("config.toml");

    // Check if config.toml exists
    if !config_path.exists() {
        eprintln!("❌ config.toml not found. Please run `sgdkx install` first.");
        std::process::exit(1);
    }

    let text = fs::read_to_string(&config_path).expect("config.toml read failed");
    let doc = text
        .parse::<DocumentMut>()
        .expect("TOML parse failed");
    let (sgdk_path_str, _) = get_sgdk_config(&doc);
    let sgdk_path = Path::new(sgdk_path_str.unwrap_or_else(|| {
        eprintln!("SGDK path not found in config.toml.");
        std::process::exit(1);
    }));

    let dest_path = Path::new(name);
    if dest_path.exists() {
        eprintln!("❌ '{}' already exists.", name);
        std::process::exit(1);
    }

    // テンプレート選択（--template 指定 / TTYで対話 / 非TTYはエラー）
    let template_path = select_template(sgdk_path, args.template.as_deref());

    println!("📁 Creating project from SGDK template: '{}'", name);

    let mut opts = fs_extra::dir::CopyOptions::new();
    opts.copy_inside = true;
    fs_extra::dir::copy(&template_path, dest_path, &opts).expect("Template copy failed");

    println!("✅ Project '{}' created!", name);

    // Create .clangd configuration file
    create_clangd_config(dest_path);

    // Create .vscode/c_cpp_properties.json
    create_vscode_config(dest_path);

    // Create .gitignore
    create_gitignore(dest_path);

    // Create the Makefile (native: makefile.gen + toolchain/JRE on PATH)
    let toolchain = get_toolchain_path(&doc);
    let jre = get_jre_path(&doc);
    create_makefile(
        dest_path,
        sgdk_path,
        toolchain.as_deref().map(Path::new),
        jre.as_deref().map(Path::new),
    );

    // Generate compile_commands.json (no external compiledb dependency).
    // base_make_command sets up PATH so `make -nwB` resolves (esp. on Windows).
    generate_compile_commands(&doc, dest_path);
}

/// Collect every template (a dir under SGDK/sample containing `src/`), keyed by its path
/// relative to `sample/` (e.g. "basics/hello-world"), sorted by that key.
fn collect_templates(sgdk_path: &Path) -> Vec<(String, PathBuf)> {
    fn walk(base: &Path, rel: String, out: &mut Vec<(String, PathBuf)>) {
        if base.join("src").exists() {
            out.push((rel.clone(), base.to_path_buf()));
        }
        if let Ok(entries) = std::fs::read_dir(base) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    let name = entry.file_name().to_string_lossy().to_string();
                    let new_rel = if rel.is_empty() { name } else { format!("{rel}/{name}") };
                    walk(&path, new_rel, out);
                }
            }
        }
    }
    let mut templates = Vec::new();
    walk(&sgdk_path.join("sample"), String::new(), &mut templates);
    templates.sort_by(|a, b| a.0.cmp(&b.0));
    templates
}

/// Resolve the template directory: explicit `--template <name>` wins; otherwise an interactive
/// pick on a terminal; otherwise (non-interactive, no flag) an error listing the templates.
fn select_template(sgdk_path: &Path, explicit: Option<&str>) -> PathBuf {
    let templates = collect_templates(sgdk_path);
    if templates.is_empty() {
        eprintln!("❌ No templates found in {}", sgdk_path.join("sample").display());
        std::process::exit(1);
    }

    let list_available = || {
        eprintln!("Available templates:");
        for (rel, _) in &templates {
            eprintln!("  {rel}");
        }
    };

    if let Some(name) = explicit {
        return match templates.iter().find(|(rel, _)| rel == name) {
            Some((rel, path)) => {
                println!("Using template: {rel}");
                path.clone()
            }
            None => {
                eprintln!("❌ template '{name}' not found.");
                list_available();
                std::process::exit(1);
            }
        };
    }

    if !std::io::stdin().is_terminal() {
        eprintln!("❌ no template selected. Re-run with --template <name> (required when non-interactive).");
        list_available();
        std::process::exit(1);
    }

    use dialoguer::{Select, theme::ColorfulTheme};
    let items: Vec<&str> = templates.iter().map(|(rel, _)| rel.as_str()).collect();
    let selection = Select::with_theme(&ColorfulTheme::default())
        .with_prompt("Select a project template from SGDK/sample (Esc to cancel)")
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

/// Generate compile_commands.json by parsing `make -nwB` output (a make dry-run),
/// so clangd / IntelliSense work without an external `compiledb` dependency.
/// The compile flags are deterministic (from SGDK's common.mk).
pub fn generate_compile_commands(doc: &DocumentMut, project_path: &Path) {
    println!("🔧 Generating compile_commands.json...");

    let output = match crate::commands::make::make_command(doc, &["-nwB"])
        .current_dir(project_path)
        .output()
    {
        Ok(o) => o,
        Err(e) => {
            eprintln!("⚠️  could not run make for compile_commands.json: {}", e);
            return;
        }
    };

    let stdout = String::from_utf8_lossy(&output.stdout);
    let dir = project_path
        .canonicalize()
        .unwrap_or_else(|_| project_path.to_path_buf());
    let dir_str = dir.to_string_lossy().replace(r"\\?\", "");

    let mut entries: Vec<serde_json::Value> = Vec::new();
    for line in stdout.lines() {
        let line = line.trim();
        // keep real C compiles: gcc with `-c <src.c>`, excluding dependency-gen (`-E`)
        if !(line.contains("gcc") && line.contains(" -c ") && !line.contains(" -E")) {
            continue;
        }
        let tokens: Vec<&str> = line.split_whitespace().collect();
        let file = tokens
            .iter()
            .position(|t| *t == "-c")
            .and_then(|i| tokens.get(i + 1))
            .copied();
        let file = match file {
            Some(f) if f.ends_with(".c") => f,
            _ => continue,
        };
        entries.push(serde_json::json!({
            "directory": dir_str,
            "command": line,
            "file": file,
        }));
    }

    if entries.is_empty() {
        eprintln!("⚠️  no compile commands captured; compile_commands.json not written");
        return;
    }
    let json = serde_json::to_string_pretty(&entries).unwrap();
    match fs::write(project_path.join("compile_commands.json"), json) {
        Ok(_) => println!("✅ compile_commands.json generated ({} entries)", entries.len()),
        Err(e) => eprintln!("⚠️  failed to write compile_commands.json: {}", e),
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

/// config.tomlのtoolchainインラインテーブルからpathを取得（Windowsではなし）
pub fn get_toolchain_path(doc: &DocumentMut) -> Option<String> {
    doc.get("toolchain")
        .and_then(|v| v.as_inline_table())
        .and_then(|tbl| tbl.get("path"))
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
}

/// config.tomlのjreインラインテーブルからpathを取得（無ければsystem Java）
pub fn get_jre_path(doc: &DocumentMut) -> Option<String> {
    doc.get("jre")
        .and_then(|v| v.as_inline_table())
        .and_then(|tbl| tbl.get("path"))
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
}

pub fn create_clangd_config(project_path: &Path) {
    println!("📄 Creating .clangd configuration file...");

    let clangd_content = r#"# Configuration for using clangd with SGDK projects in Zed Editor (adjustments for GCC-based code)
CompileFlags:
  Add:
    - '-DSGDK_GCC'
    - '-include'
    - 'types.h'
    - '-std=c17'
  Remove:
    - '-ffat-lto-objects'
    - '-externally_visible'
    - '-f*'
    - '-m68000'
Diagnostics:
  Suppress:
    - 'main_arg_wrong'
    - '-Wunknown-attributes'
"#;

    let clangd_path = project_path.join(".clangd");
    fs::write(clangd_path, clangd_content).expect("Failed to create .clangd file");
    println!("✅ .clangd configuration file created");
}

pub fn create_vscode_config(project_path: &Path) {
    println!("📄 Creating .vscode/c_cpp_properties.json...");

    let vscode_dir = project_path.join(".vscode");
    if !vscode_dir.exists() {
        fs::create_dir_all(&vscode_dir).expect("Failed to create .vscode directory");
    }

    let cpp_properties_content = r#"{
    "configurations": [
      {
        "name": "sgdk",
        "cStandard": "c17",
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
    println!("✅ VS Code C++ configuration file created");
}

pub fn create_gitignore(project_path: &Path) {
    println!("📄 Creating .gitignore file...");

    let gitignore_content = r#"/compile_commands.json
/.cache
/out
/res/**/*.h
/res/**/*.rs
/Makefile
"#;

    let gitignore_path = project_path.join(".gitignore");
    fs::write(gitignore_path, gitignore_content).expect("Failed to create .gitignore file");
    println!("✅ .gitignore file created");
}

/// Path to a forward-slash string with the Windows `//?/` verbatim prefix stripped.
fn unixify(p: &Path) -> String {
    let s = p.to_string_lossy().replace('\\', "/");
    s.strip_prefix("//?/").map(|s| s.to_string()).unwrap_or(s)
}

pub fn create_makefile(
    project_path: &Path,
    sgdk_path: &Path,
    toolchain_path: Option<&Path>,
    jre_path: Option<&Path>,
) {
    println!("📄 Creating Makefile...");

    let gdk = unixify(sgdk_path);
    println!("Using SGDK path: {}", gdk);

    // On Unix the bundled JRE (for rescomp/sizebnd), the native gcc toolchain, and
    // SGDK's native tools (convsym/sjasm/bintos/mac68k, resolved bare by common.mk)
    // must be on PATH; prepend them for make's recipes only (no global PATH change).
    // On Windows SGDK uses its bundled bin/*.exe toolchain, so no PATH line is needed.
    let path_line = match toolchain_path {
        Some(tc) => {
            let mut dirs = Vec::new();
            if let Some(jre) = jre_path {
                dirs.push(unixify(&jre.join("bin")));
            }
            dirs.push(unixify(&tc.join("bin")));
            dirs.push("$(GDK)/bin".to_string());
            format!("export PATH := {}:$(PATH)\n", dirs.join(":"))
        }
        None => String::new(),
    };

    let makefile_content = format!(
        r#"# SGDK Makefile - generated by sgdkx
# Note: this file is in .gitignore to avoid committing personal paths.
#
# usage:
#   make           # build the project (release)
#   make debug     # build with debug symbols
#   make clean     # remove build artifacts

GDK = {gdk}
{path_line}include $(GDK)/makefile.gen
"#,
    );

    let makefile_path = project_path.join("Makefile");
    fs::write(makefile_path, makefile_content).expect("Failed to create Makefile");
    println!("✅ Makefile created successfully");
}
