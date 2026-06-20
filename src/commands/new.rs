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

    // Load config (shared loader; exits with an install hint if missing).
    let doc = crate::commands::make::load_config();
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

    // Create .vscode/launch.json + tasks.json (source-level gdb debugging)
    create_vscode_debug_config(dest_path);

    // Create .gitignore
    create_gitignore(dest_path);

    // Create the Makefile (portable + committable; `sgdkx make` sets GDK + the toolchain PATH)
    create_makefile(dest_path);

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
        // The SGDK compile rule is `... -c <src> -o <obj>`. Take the argument BETWEEN ` -c `
        // and ` -o ` so source paths containing spaces survive (split_whitespace would shatter
        // them and capture only the first fragment).
        let file = match line
            .split_once(" -c ")
            .and_then(|(_, rest)| rest.split_once(" -o "))
            .map(|(f, _)| f.trim())
        {
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
    - '-std=gnu17'
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

    // cStandard: SGDK builds with gcc's default (no -std) = gnu17 (C17 + GNU extensions) and
    //   relies on -fms-extensions/statement-exprs, so `gnu17` (not `c17`) avoids false squiggles.
    // intelliSenseMode: m68k is ILP32 (int/long/pointer = 4 bytes); `gcc-x86` matches those sizes
    //   far better than `gcc-x64` (8-byte pointers). cpptools has no m68k mode; for files in
    //   compile_commands.json the real m68k-elf-gcc invocation is used regardless.
    let cpp_properties_content = r#"{
    "configurations": [
      {
        "name": "sgdk",
        "cStandard": "gnu17",
        "intelliSenseMode": "gcc-x86",
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

/// Write VS Code source-level debug configs (launch.json + tasks.json).
///
/// Portable & committable: paths are home-relative (`${userHome}`) and BlastEm is
/// launched via `sgdkx blastem`, so there are no machine-specific paths. tasks.json
/// builds a -O0 debug ROM (so breakpoints/locals are reliable — see the OPT note in
/// the Makefile), starts BlastEm as a gdb server on localhost:1234, and launch.json
/// connects via gdb. sourceFileMap / `set substitute-path` remap SGDK's CI build path
/// (baked into the prebuilt libmd debug info) to the local install, so you can step
/// into SGDK source too.
///
/// Requires the gdb-capable BlastEm — the patched build that honors `-D` and
/// BLASTEM_GDB_PORT. `sgdkx install` provides it; if your BlastEm predates gdb
/// support, update it (otherwise F5 connects to nothing).
pub fn create_vscode_debug_config(project_path: &Path) {
    println!("📄 Creating .vscode/launch.json + tasks.json (gdb debugging)...");

    let vscode_dir = project_path.join(".vscode");
    if !vscode_dir.exists() {
        fs::create_dir_all(&vscode_dir).expect("Failed to create .vscode directory");
    }

    // gdb (m68k-elf-gdb) is launched directly by cppdbg, so it needs a real path;
    // BlastEm goes through `sgdkx blastem` so we don't hardcode its location.
    // The sourceFileMap "from" is the fixed path where sgdk-native-builds compiles
    // SGDK in CI; "to" is the local install. If CI ever moves, this just falls back
    // to "no source" (harmless).
    let launch_json = r#"{
  // Source-level debugging of the ROM in (patched) BlastEm via m68k-elf-gdb.
  // Set breakpoints in src/*.c, press F5, then Continue (▶) to reach them.
  // Drive with breakpoints + Continue; only "Step Into" your OWN functions, and
  // don't "Step Over" the SYS_doVBlankProcess() line (frame-sync; use Continue).
  "version": "0.2.0",
  "configurations": [
    {
      "name": "Debug ROM (BlastEm) — cpptools",
      "type": "cppdbg",
      "request": "launch",
      "program": "${workspaceFolder}/out/debug/rom.out",
      "cwd": "${workspaceFolder}",
      "MIMode": "gdb",
      "miDebuggerPath": "${userHome}/.sgdkx/data/m68k-elf-gdb/bin/m68k-elf-gdb",
      "miDebuggerServerAddress": "localhost:1234",
      "stopAtConnect": false,
      "externalConsole": false,
      "preLaunchTask": "blastem-gdb",
      "sourceFileMap": {
        "/Users/runner/work/sgdk-native-builds/sgdk-native-builds/SGDK": "${userHome}/.sgdkx/data/SGDK"
      },
      "setupCommands": [
        { "description": "break at main", "text": "-break-insert main", "ignoreFailures": true }
      ]
    },
    {
      "name": "Debug ROM (BlastEm) — Native Debug",
      "type": "gdb",
      "request": "attach",
      "executable": "${workspaceFolder}/out/debug/rom.out",
      "target": "localhost:1234",
      "remote": true,
      "cwd": "${workspaceFolder}",
      "gdbpath": "${userHome}/.sgdkx/data/m68k-elf-gdb/bin/m68k-elf-gdb",
      "valuesFormatting": "parseText",
      "stopAtConnect": false,
      "preLaunchTask": "blastem-gdb",
      "autorun": [
        "set substitute-path /Users/runner/work/sgdk-native-builds/sgdk-native-builds/SGDK ${userHome}/.sgdkx/data/SGDK",
        "break main"
      ]
    }
  ]
}
"#;

    let tasks_json = r#"{
  // build-debug : -O0 debug ROM with DWARF (clean stepping); see the Makefile OPT note.
  // blastem-gdb : runs the patched BlastEm as a gdb server on TCP localhost:1234.
  //   It blocks until the debugger connects; SDL_AUDIODRIVER=dummy + BLASTEM_NO_GUI
  //   keep headless launches from stalling on a CoreAudio error dialog.
  "version": "2.0.0",
  "tasks": [
    {
      "label": "build-debug",
      "type": "shell",
      "command": "sgdkx make clean-debug && sgdkx make debug OPT=-O0",
      "options": { "cwd": "${workspaceFolder}" },
      "group": "build",
      "problemMatcher": ["$gcc"]
    },
    {
      "label": "blastem-gdb",
      "type": "shell",
      "command": "sgdkx",
      "args": ["blastem", "${workspaceFolder}/out/debug/rom.bin", "-D"],
      "options": {
        "env": {
          "BLASTEM_GDB_PORT": "1234",
          "BLASTEM_NO_GUI": "1",
          "SDL_AUDIODRIVER": "dummy"
        }
      },
      "dependsOn": "build-debug",
      "isBackground": true,
      "problemMatcher": {
        "pattern": { "regexp": "^____no_problems____$" },
        "background": {
          "activeOnStart": true,
          "beginsPattern": ".",
          "endsPattern": "Waiting for GDB connection"
        }
      }
    }
  ]
}
"#;

    fs::write(vscode_dir.join("launch.json"), launch_json).expect("Failed to create launch.json");
    fs::write(vscode_dir.join("tasks.json"), tasks_json).expect("Failed to create tasks.json");
    println!("✅ VS Code debug configuration created (gdb via patched BlastEm)");
}

pub fn create_gitignore(project_path: &Path) {
    println!("📄 Creating .gitignore file...");

    // Makefile is now portable (no personal paths) and meant to be committed, so it is NOT
    // ignored. compile_commands.json holds absolute compile commands -> still ignored
    // (regenerate with `sgdkx compile-commands`).
    let gitignore_content = r#"/compile_commands.json
/.cache
/out
/res/**/*.h
/res/**/*.rs
"#;

    let gitignore_path = project_path.join(".gitignore");
    fs::write(gitignore_path, gitignore_content).expect("Failed to create .gitignore file");
    println!("✅ .gitignore file created");
}

/// Write a portable, committable Makefile (no machine-specific paths).
///
/// `sgdkx make` exports `GDK` (the exact resolved SGDK path) and the toolchain PATH, so the
/// build is correct on every platform. The `?=` default lets plain `make` work on a standard
/// Unix install; override the install location with `GDK=/path make`. Because the install
/// dir is now unified to `~/.sgdkx/data` on all platforms, the default is portable.
pub fn create_makefile(project_path: &Path) {
    println!("📄 Creating Makefile...");

    let makefile_content = r#"# SGDK Makefile — generated by sgdkx (safe to commit; no personal paths).
#
# Recommended: build with `sgdkx make` (it sets GDK and the toolchain PATH).
# Plain `make` also works when sgdkx is installed at the default location and the
# toolchain is on PATH. Override the install location with `GDK=/path make`.
#
# usage:
#   sgdkx make                 # build the project (release)
#   sgdkx make debug           # build with debug symbols (-O1, SGDK default)
#   sgdkx make debug OPT=-O0   # debug build, unoptimized (best for gdb stepping)
#   sgdkx make debug OPT=-Og   # debug build, lighter optimization
#   sgdkx make clean           # remove build artifacts
GDK ?= $(HOME)/.sgdkx/data/SGDK
include $(GDK)/makefile.gen

# Optional override of the debug optimization level for gdb stepping, WITHOUT
# touching SGDK. The debug build is -O1 by default; with -O1 gcc reorders code so
# stepping jumps around, some lines hold no code, and locals show "<optimized out>".
# `make debug OPT=-O0` (or OPT=-Og) swaps that out for a debug-friendly level.
# NOTE: make rebuilds on file timestamps, not flag changes — run `make clean-debug`
# when switching OPT levels (objects under out/debug are shared).
ifdef OPT
override CFLAGS := $(filter-out -O1,$(CFLAGS)) $(OPT)
endif
"#;

    let makefile_path = project_path.join("Makefile");
    fs::write(makefile_path, makefile_content).expect("Failed to create Makefile");
    println!("✅ Makefile created");
}
