use clap::Parser;
use std::fs;
use std::io::IsTerminal;
use std::path::{Path, PathBuf};

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

    if !crate::path::is_installed() {
        eprintln!("❌ SGDK not installed. Please run `sgdkx install` first.");
        std::process::exit(1);
    }
    let sgdk_path = crate::path::sgdk_dir();

    let dest_path = Path::new(name);
    if dest_path.exists() {
        eprintln!("❌ '{}' already exists.", name);
        std::process::exit(1);
    }

    // テンプレート選択（--template 指定 / TTYで対話 / 非TTYはエラー）
    let template_path = select_template(&sgdk_path, args.template.as_deref());

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
    generate_compile_commands(dest_path);
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
pub fn generate_compile_commands(project_path: &Path) {
    println!("🔧 Generating compile_commands.json...");

    let output = match crate::commands::make::make_command(&["-nwB"])
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
    // includePath/defines: fallback for files NOT in compile_commands.json — notably the
    //   rescomp-generated res/*.h, which `#include <genesis.h>` ($GDK/inc): without it cpptools
    //   shows "cannot open source file genesis.h".
    // compilerPath: the bundled m68k-elf-gcc. cpptools needs an explicit compiler to query for
    //   the toolchain system headers + intrinsics (e.g. <stdint.h>'s uint8_t); without it, even
    //   .c files in compile_commands.json get "identifier uint8_t is undefined" (the compile
    //   command names a bare `m68k-elf-gcc`, which cpptools won't run on its own). This is the
    //   macOS/Linux toolchain path; on Windows gcc lives in SGDK/bin and compile_commands carries
    //   its absolute path, so a stale path here is only a harmless warning. Home-relative.
    let cpp_properties_content = r#"{
    "configurations": [
      {
        "name": "sgdk",
        "compileCommands": "${workspaceFolder}/compile_commands.json",
        "compilerPath": "${userHome}/.sgdkx/data/m68k-elf-toolchain/bin/m68k-elf-gcc",
        "cStandard": "gnu17",
        "intelliSenseMode": "gcc-x86",
        "includePath": [
          "${workspaceFolder}/**",
          "${userHome}/.sgdkx/data/SGDK/inc",
          "${userHome}/.sgdkx/data/SGDK/res"
        ],
        "defines": [ "SGDK_GCC" ]
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
/// connects via gdb. The default configs link the lean libmd.a (debug your own code,
/// small ROM); the "+ SGDK source" config rebuilds with SGDK_DEBUG=1 and, via
/// sourceFileMap / `set substitute-path` remapping SGDK's CI build path (baked into
/// libmd_debug.a) to the local install, lets you step into SGDK source too.
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
  // Drive with breakpoints + Continue; don't "Step Over" the SYS_doVBlankProcess()
  // line (frame-sync; use Continue). Pick a config from the Run and Debug dropdown:
  //   "Debug ROM (BlastEm)"               debug your own code (lean libmd.a; small/fast)
  //   "Debug ROM (BlastEm) + SGDK source" also step into SGDK functions (SGDK_DEBUG=1)
  "version": "0.2.0",
  "configurations": [
    {
      "name": "Debug ROM (BlastEm)",
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
      "name": "Debug ROM (BlastEm) + SGDK source",
      "type": "cppdbg",
      "request": "launch",
      "program": "${workspaceFolder}/out/debug/rom.out",
      "cwd": "${workspaceFolder}",
      "MIMode": "gdb",
      "miDebuggerPath": "${userHome}/.sgdkx/data/m68k-elf-gdb/bin/m68k-elf-gdb",
      "miDebuggerServerAddress": "localhost:1234",
      "stopAtConnect": false,
      "externalConsole": false,
      "preLaunchTask": "blastem-gdb-sgdk",
      "sourceFileMap": {
        "/Users/runner/work/sgdk-native-builds/sgdk-native-builds/SGDK": "${userHome}/.sgdkx/data/SGDK"
      },
      "setupCommands": [
        { "description": "break at main", "text": "-break-insert main", "ignoreFailures": true }
      ]
    }
  ]
}
"#;

    let tasks_json = r#"{
  // build-debug      : -O0 debug ROM with DWARF, lean libmd.a (debug your code; small ROM).
  // build-debug-sgdk : same + SGDK_DEBUG=1 -> libmd_debug.a, so you can step into SGDK
  //                    source too (larger ROM, slower link; needs SGDK >= 2.10).
  // blastem-gdb[-sgdk] : run the patched BlastEm as a gdb server on TCP localhost:1234 after
  //   the matching build. Blocks until the debugger connects; SDL_AUDIODRIVER=dummy +
  //   BLASTEM_NO_GUI keep headless launches from stalling on a CoreAudio error dialog.
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
      "label": "build-debug-sgdk",
      "type": "shell",
      "command": "sgdkx make clean-debug && sgdkx make debug OPT=-O0 SGDK_DEBUG=1",
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
    },
    {
      "label": "blastem-gdb-sgdk",
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
      "dependsOn": "build-debug-sgdk",
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

/// Write the project Makefile. It carries no machine-specific paths — `GDK ?=` defaults to the
/// unified `~/.sgdkx/data` install and `sgdkx make` sets `GDK` + the build-tool PATH, so the
/// same file builds on every platform.
pub fn create_makefile(project_path: &Path) {
    println!("📄 Creating Makefile...");

    let makefile_content = r#"# SGDK project Makefile — generated by sgdkx.
#
#   sgdkx make                    # build (release)
#   sgdkx make debug              # debug build: -O0 + symbols, lean libmd.a
#   sgdkx make debug OPT=-Og      # debug build, lighter optimization
#   sgdkx make debug SGDK_DEBUG=1 # also step into SGDK source (needs SGDK >= 2.10)
#   sgdkx make clean              # remove build artifacts
GDK ?= $(HOME)/.sgdkx/data/SGDK
include $(GDK)/makefile.gen

# Tune `make debug` for source-level debugging (SGDK's debug build is -O1 +
# libmd_debug.a): -O0 keeps stepping/locals reliable; the lean libmd.a debugs your
# code only and keeps the ROM small (SGDK_DEBUG=1 to step into SGDK instead).
# Switching OPT/SGDK_DEBUG needs `make clean-debug` (make keys on timestamps).
ifeq ($(BUILD_TYPE),debug)
OPT ?= -O0
override CFLAGS := $(filter-out -O1,$(CFLAGS)) $(OPT)
ifndef SGDK_DEBUG
override LIBMD := $(LIB)/libmd.a
endif
endif
"#;

    let makefile_path = project_path.join("Makefile");
    fs::write(makefile_path, makefile_content).expect("Failed to create Makefile");
    println!("✅ Makefile created");
}
