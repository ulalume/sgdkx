use clap::{Parser, Subcommand};
use dirs::config_dir;
use fs_extra::dir::{CopyOptions, copy};
use reqwest::blocking::get;
use sevenz_rust;
use std::fs;
use std::io::copy as io_copy;
use std::os::unix::fs::symlink;
use std::path::{Path, PathBuf};
use std::process::Command;
use tempfile::NamedTempFile;
use toml_edit::{Document, value};
use which::which;
use zip::ZipArchive;

// 多言語化の初期化
rust_i18n::i18n!("locales");

/// SGDK support CLI tool for Mega Drive / Genesis game dev
#[derive(Parser)]
#[command(name = "sgdktool")]
#[command(version = "0.1.0")]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    Setup {
        /// Directory to clone SGDK into (defaults to config directory)
        #[arg(long)]
        dir: Option<String>,

        /// Branch name to clone
        #[arg(long, default_value = "master")]
        branch: String,
    },

    /// Setup emulator for running ROM files
    SetupEmu {
        /// Emulator to setup (gens or blastem)
        #[arg(default_value = "gens")]
        emulator: String,

        /// Directory to install emulator (defaults to config directory)
        #[arg(long)]
        dir: Option<String>,
    },

    New {
        /// Project name (will be created as a directory)
        name: String,
    },

    /// Build project using make
    Make {
        /// Project directory (defaults to current directory)
        #[arg(long, default_value = ".")]
        project: String,

        /// Additional options to pass to make
        #[arg(last = true)]
        extra: Vec<String>,
    },

    /// Run ROM file with emulator
    Run {
        /// Emulator to use (gens or blastem, defaults to available emulator)
        emulator: Option<String>,

        /// ROM file path (defaults to out/rom.bin)
        #[arg(long, default_value = "out/rom.bin")]
        rom: String,
    },

    /// Uninstall SGDK installation and configuration
    Uninstall {
        /// Remove only configuration (keep SGDK installation)
        #[arg(long)]
        config_only: bool,
    },
}

fn main() {
    // ロケールを設定
    init_locale();

    // 多言語化対応のCLI作成
    let cli = create_localized_cli();

    match cli.command {
        Some(cmd) => match cmd {
            Commands::Setup { dir, branch } => {
                setup_sgdk(dir.as_deref(), &branch);
            }
            Commands::SetupEmu { emulator, dir } => {
                setup_emulator(&emulator, dir.as_deref());
            }
            Commands::New { name } => {
                create_project(&name);
            }
            Commands::Make { project, extra } => {
                build_project(&project, extra);
            }
            Commands::Run { emulator, rom } => {
                run_emulator(emulator.as_deref(), &rom);
            }
            Commands::Uninstall { config_only } => {
                uninstall_sgdk(config_only);
            }
        },
        None => {
            // コマンドが指定されなかったときに実行したいロジック
            run_doctor_and_info();
        }
    }
}

fn init_locale() {
    // システムのロケールを取得
    let locale = std::env::var("LANG")
        .or_else(|_| std::env::var("LC_ALL"))
        .unwrap_or_else(|_| "en".to_string());

    // 日本語ロケールの場合は "ja" を設定
    if locale.starts_with("ja") {
        rust_i18n::set_locale("ja");
    } else {
        rust_i18n::set_locale("en");
    }
}

fn create_localized_cli() -> Cli {
    use clap::Command;

    // ロケールチェック（ライフタイムエラーを避けるため条件分岐を使用）
    let is_japanese = rust_i18n::locale().to_string() == "ja";

    let app = Command::new("sgdktool")
        .version("0.1.0")
        .about(if is_japanese {
            "SGDKサポートCLIツール"
        } else {
            "A CLI tool for SGDK-based development"
        })
        .subcommand(
            Command::new("setup")
                .about(if is_japanese {
                    "SGDKをセットアップ（クローンとパス登録）"
                } else {
                    "Setup SGDK (clone and register path)"
                })
                .arg(clap::Arg::new("dir").long("dir").help(if is_japanese {
                    "SGDKをクローンするディレクトリ（省略時は設定ディレクトリ）"
                } else {
                    "Directory to clone SGDK into (defaults to config directory)"
                }))
                .arg(
                    clap::Arg::new("branch")
                        .long("branch")
                        .default_value("master")
                        .help(if is_japanese {
                            "クローンするブランチ名"
                        } else {
                            "Branch name to clone"
                        }),
                ),
        )
        .subcommand(
            Command::new("setup-emu")
                .about(if is_japanese {
                    "ROMファイル実行用のエミュレータをセットアップ"
                } else {
                    "Setup emulator for running ROM files"
                })
                .arg(
                    clap::Arg::new("emulator")
                        .default_value("gens")
                        .help(if is_japanese {
                            "セットアップするエミュレータ (gens または blastem)"
                        } else {
                            "Emulator to setup (gens or blastem)"
                        }),
                )
                .arg(clap::Arg::new("dir").long("dir").help(if is_japanese {
                    "エミュレータをインストールするディレクトリ（省略時は設定ディレクトリ）"
                } else {
                    "Directory to install emulator (defaults to config directory)"
                })),
        )
        .subcommand(
            Command::new("new")
                .about(if is_japanese {
                    "SGDKテンプレートから新しいプロジェクトを作成"
                } else {
                    "Create new project from SGDK template"
                })
                .arg(clap::Arg::new("name").required(true).help(if is_japanese {
                    "プロジェクト名（ディレクトリとして作成されます）"
                } else {
                    "Project name (will be created as a directory)"
                })),
        )
        .subcommand(
            Command::new("make")
                .about(if is_japanese {
                    "makeを使ってプロジェクトをビルド"
                } else {
                    "Build project using make"
                })
                .arg(
                    clap::Arg::new("project")
                        .long("project")
                        .default_value(".")
                        .help(if is_japanese {
                            "プロジェクトディレクトリ（省略時はカレントディレクトリ）"
                        } else {
                            "Project directory (defaults to current directory)"
                        }),
                )
                .arg(
                    clap::Arg::new("extra")
                        .trailing_var_arg(true)
                        .num_args(0..)
                        .help(if is_japanese {
                            "makeに渡す追加オプション"
                        } else {
                            "Additional options to pass to make"
                        }),
                ),
        )
        .subcommand(
            Command::new("run")
                .about(if is_japanese {
                    "エミュレータでROMファイルを実行"
                } else {
                    "Run ROM file with emulator"
                })
                .arg(clap::Arg::new("emulator").help(if is_japanese {
                    "使用するエミュレータ (gens または blastem、省略時は利用可能なエミュレータ)"
                } else {
                    "Emulator to use (gens or blastem, defaults to available emulator)"
                }))
                .arg(
                    clap::Arg::new("rom")
                        .long("rom")
                        .default_value("out/rom.bin")
                        .help(if is_japanese {
                            "ROMファイルのパス（省略時は out/rom.bin）"
                        } else {
                            "ROM file path (defaults to out/rom.bin)"
                        }),
                ),
        )
        .subcommand(
            Command::new("uninstall")
                .about(if is_japanese {
                    "SGDKインストールと設定をアンインストール"
                } else {
                    "Uninstall SGDK installation and configuration"
                })
                .arg(
                    clap::Arg::new("config-only")
                        .long("config-only")
                        .action(clap::ArgAction::SetTrue)
                        .help(if is_japanese {
                            "設定のみ削除（SGDKインストールは保持）"
                        } else {
                            "Remove only configuration (keep SGDK installation)"
                        }),
                ),
        );

    let matches = app.get_matches();

    // マッチした結果をCli構造体に変換
    match matches.subcommand() {
        Some(("setup", sub_matches)) => Cli {
            command: Some(Commands::Setup {
                dir: sub_matches.get_one::<String>("dir").cloned(),
                branch: sub_matches.get_one::<String>("branch").unwrap().clone(),
            }),
        },
        Some(("new", sub_matches)) => Cli {
            command: Some(Commands::New {
                name: sub_matches.get_one::<String>("name").unwrap().clone(),
            }),
        },
        Some(("make", sub_matches)) => Cli {
            command: Some(Commands::Make {
                project: sub_matches.get_one::<String>("project").unwrap().clone(),
                extra: sub_matches
                    .get_many::<String>("extra")
                    .unwrap_or_default()
                    .map(|s| s.clone())
                    .collect(),
            }),
        },
        Some(("setup-emu", sub_matches)) => Cli {
            command: Some(Commands::SetupEmu {
                emulator: sub_matches.get_one::<String>("emulator").unwrap().clone(),
                dir: sub_matches.get_one::<String>("dir").cloned(),
            }),
        },
        Some(("run", sub_matches)) => Cli {
            command: Some(Commands::Run {
                emulator: sub_matches.get_one::<String>("emulator").cloned(),
                rom: sub_matches.get_one::<String>("rom").unwrap().clone(),
            }),
        },
        Some(("uninstall", sub_matches)) => Cli {
            command: Some(Commands::Uninstall {
                config_only: sub_matches.get_flag("config-only"),
            }),
        },
        _ => Cli { command: None },
    }
}

fn setup_sgdk(dir: Option<&str>, branch: &str) {
    if which("git").is_err() {
        eprintln!("{}", rust_i18n::t!("git_not_found"));
        std::process::exit(1);
    }

    // デフォルトディレクトリを設定ディレクトリ配下に設定
    let target_dir = if let Some(custom_dir) = dir {
        PathBuf::from(custom_dir)
    } else {
        config_dir()
            .expect("Failed to get config directory")
            .join("sgdktool")
            .join("SGDK")
    };

    // デフォルトパスを使用する場合は、ユーザーに通知
    if dir.is_none() {
        if rust_i18n::locale().to_string() == "ja" {
            println!(
                "📁 デフォルト設定ディレクトリを使用: {}",
                target_dir.display()
            );
        } else {
            println!(
                "📁 Using default config directory: {}",
                target_dir.display()
            );
        }
    }
    if target_dir.exists() {
        println!("{}", rust_i18n::t!("sgdk_exists_overwrite"));
        use std::io::{self, Write};
        print!("{}", rust_i18n::t!("prompt"));
        io::stdout().flush().unwrap();

        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();
        let input = input.trim().to_lowercase();

        if input != "y" && input != "" {
            println!("{}", rust_i18n::t!("operation_cancelled"));
            std::process::exit(0);
        }

        // Only create config file if user chooses not to overwrite
        println!("{}", rust_i18n::t!("saving_config"));
        let config_dir = config_dir()
            .expect("Failed to get config directory")
            .join("sgdktool");
        fs::create_dir_all(&config_dir).expect("Failed to create config directory");
        let config_path = config_dir.join("config.toml");

        let mut doc = if config_path.exists() {
            let text =
                fs::read_to_string(&config_path).expect(&rust_i18n::t!("config_read_failed"));
            text.parse::<Document>()
                .expect(&rust_i18n::t!("toml_parse_failed"))
        } else {
            Document::new()
        };
        let abs_path = target_dir
            .canonicalize()
            .expect("Failed to get absolute path");
        doc["sgdk"]["path"] = value(abs_path.to_str().unwrap());
        doc["sgdk"]["branch"] = value(branch);

        fs::write(&config_path, doc.to_string()).expect("Failed to write config.toml");
        println!("{}", rust_i18n::t!("config_only_created"));
        return;
    }

    println!("{}", rust_i18n::t!("cloning_sgdk"));
    // 親ディレクトリが存在しない場合は作成
    if let Some(parent) = target_dir.parent() {
        fs::create_dir_all(parent).expect("Failed to create parent directory");
    }

    let status = Command::new("git")
        .args([
            "clone",
            "--branch",
            branch,
            "https://github.com/Stephane-D/SGDK",
            target_dir.to_str().unwrap(),
        ])
        .status()
        .expect("git clone failed");

    if !status.success() {
        eprintln!("{}", rust_i18n::t!("git_clone_failed"));
        std::process::exit(1);
    }

    println!("{}", rust_i18n::t!("saving_config"));
    let config_dir = config_dir()
        .expect("Failed to get config directory")
        .join("sgdktool");
    fs::create_dir_all(&config_dir).expect("Failed to create config directory");
    let config_path = config_dir.join("config.toml");

    let mut doc = if config_path.exists() {
        let text = fs::read_to_string(&config_path).expect(&rust_i18n::t!("config_read_failed"));
        text.parse::<Document>()
            .expect(&rust_i18n::t!("toml_parse_failed"))
    } else {
        Document::new()
    };
    let abs_path = target_dir
        .canonicalize()
        .expect("Failed to get absolute path");
    doc["sgdk"]["path"] = value(abs_path.to_str().unwrap());
    doc["sgdk"]["branch"] = value(branch);

    fs::write(&config_path, doc.to_string()).expect("Failed to write config.toml");

    if !cfg!(target_os = "windows") {
        run_generate_wine(&target_dir);
    }

    println!(
        "{}",
        rust_i18n::t!("sgdk_setup_complete", path = target_dir.display())
    );
}

fn create_project(name: &str) {
    let config_path = config_dir().unwrap().join("sgdktool/config.toml");

    // Check if config.toml exists
    if !config_path.exists() {
        eprintln!("{}", rust_i18n::t!("config_not_found_for_project"));
        std::process::exit(1);
    }

    let text = fs::read_to_string(&config_path).expect(&rust_i18n::t!("config_read_failed"));
    let doc = text
        .parse::<Document>()
        .expect(&rust_i18n::t!("toml_parse_failed"));
    let sgdk_path = Path::new(doc["sgdk"]["path"].as_str().unwrap());

    let template_path = sgdk_path.join("project").join("template");
    let dest_path = Path::new(name);

    if dest_path.exists() {
        eprintln!("{}", rust_i18n::t!("project_exists", name = name));
        std::process::exit(1);
    }

    println!("{}", rust_i18n::t!("creating_project", name = name));

    let mut opts = CopyOptions::new();
    opts.copy_inside = true;
    copy(&template_path, &dest_path, &opts).expect("Template copy failed");

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

fn build_project(project: &str, extra: Vec<String>) {
    let dir = Path::new(project);
    if !dir.exists() {
        eprintln!("{}", rust_i18n::t!("project_dir_not_found"));
        std::process::exit(1);
    }

    let config_path = config_dir().unwrap().join("sgdktool/config.toml");
    let doc = fs::read_to_string(&config_path)
        .unwrap()
        .parse::<Document>()
        .unwrap();
    let sgdk_path = Path::new(doc["sgdk"]["path"].as_str().unwrap());

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

    let makefile = if cfg!(target_os = "windows") {
        effective_sgdk_path.join("makefile.gen")
    } else {
        effective_sgdk_path.join("makefile_wine.gen")
    };

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

fn run_generate_wine(sgdk_path: &Path) {
    let sgdk_bin = sgdk_path.join("bin");
    let script_url =
        "https://raw.githubusercontent.com/Franticware/SGDK_wine/master/generate_wine.sh";
    let local_script = sgdk_bin.join("generate_wine.sh");

    println!("{}", rust_i18n::t!("wine_downloading"));
    let response = reqwest::blocking::get(script_url)
        .expect("Script download failed")
        .text()
        .expect("Text retrieval failed");
    fs::write(&local_script, response).expect("Failed to write generate_wine.sh");

    println!("{}", rust_i18n::t!("wine_generating"));
    let status = Command::new("sh")
        .arg("generate_wine.sh")
        .current_dir(sgdk_path.join("bin"))
        .status()
        .expect("Failed to execute generate_wine.sh");

    if !status.success() {
        eprintln!("{}", rust_i18n::t!("wine_script_failed"));
        std::process::exit(1);
    }

    println!("{}", rust_i18n::t!("wine_wrapper_complete"));
}

fn run_doctor_and_info() {
    show_help_output();

    println!("\n{}", rust_i18n::t!("environment_check"));

    for tool in ["git", "make", "java", "compiledb"].iter() {
        check_tool(tool);
    }

    #[cfg(any(target_os = "macos", target_os = "linux"))]
    check_tool("wine");

    let config_path = config_dir().unwrap().join("sgdktool").join("config.toml");

    println!("\n{}", rust_i18n::t!("sgdk_config_info"));

    if config_path.exists() {
        let text = fs::read_to_string(&config_path).unwrap();
        let doc = text.parse::<Document>().unwrap();
        let path = doc["sgdk"]["path"].as_str().unwrap_or("Unknown");
        let branch = doc["sgdk"]["branch"].as_str().unwrap_or("Unknown");

        println!("{}", rust_i18n::t!("sgdk_path", path = path));
        println!("{}", rust_i18n::t!("branch", branch = branch));

        let commit = Command::new("git")
            .args(["rev-parse", "HEAD"])
            .current_dir(path)
            .output()
            .ok()
            .and_then(|out| String::from_utf8(out.stdout).ok())
            .unwrap_or("Unknown".to_string());
        println!("{}", rust_i18n::t!("commit_id", commit = commit.trim()));

        // === Gens/Blastem Path Info 追加 ===
        let config_base = config_dir().unwrap().join("sgdktool");

        // Gens
        let gens_path_config = doc
            .get("emulator")
            .and_then(|e| e.get("gens_path"))
            .and_then(|v| v.as_str());
        if let Some(path) = gens_path_config {
            println!("{}", rust_i18n::t!("gens_path", path = path));
        } else {
            let gens_path_opt = find_emulator_executable(&config_base, "gens");
            if let Some(gens_exe) = gens_path_opt {
                println!("{}", rust_i18n::t!("gens_path", path = gens_exe.display()));
            } else {
                println!("{}", rust_i18n::t!("gens_not_installed"));
            }
        }

        // Blastem
        let blastem_path_config = doc
            .get("emulator")
            .and_then(|e| e.get("blastem_path"))
            .and_then(|v| v.as_str());
        if let Some(path) = blastem_path_config {
            println!("{}", rust_i18n::t!("blastem_path", path = path));
        } else {
            let blastem_path_opt = find_emulator_executable(&config_base, "blastem");
            if let Some(blastem_exe) = blastem_path_opt {
                println!(
                    "{}",
                    rust_i18n::t!("blastem_path", path = blastem_exe.display())
                );
            } else {
                println!("{}", rust_i18n::t!("blastem_not_installed"));
            }
        }
        // === ここまで追加 ===
    } else {
        println!("{}", rust_i18n::t!("config_not_found"));
    }
}

fn check_tool(tool: &str) {
    match which::which(tool) {
        Ok(path) => println!(
            "{}",
            rust_i18n::t!("tool_found", tool = tool, path = path.display())
        ),
        Err(_) => println!("{}", rust_i18n::t!("tool_not_found", tool = tool)),
    }
}

fn check_compiledb_available() -> bool {
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

fn run_compiledb_make(project_path: &Path, sgdk_path: &Path) -> bool {
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
        match symlink(sgdk_path, &symlink_path) {
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

    let result = match Command::new("compiledb")
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

fn fix_compile_commands_paths(project_path: &Path, symlink_path: &Path, real_sgdk_path: &Path) {
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

fn create_clangd_config(project_path: &Path) {
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

fn create_vscode_config(project_path: &Path) {
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

fn create_gitignore(project_path: &Path) {
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

fn uninstall_sgdk(config_only: bool) {
    let config_dir = config_dir()
        .expect("Failed to get config directory")
        .join("sgdktool");

    let config_path = config_dir.join("config.toml");

    if config_only {
        // 設定ファイルのみ削除
        if config_path.exists() {
            // 確認プロンプト
            println!("{}", rust_i18n::t!("uninstall_config_confirm"));

            use std::io::{self, Write};
            print!("> ");
            io::stdout().flush().unwrap();

            let mut input = String::new();
            io::stdin().read_line(&mut input).unwrap();
            let input = input.trim().to_lowercase();

            if input == "y" || input == "yes" {
                fs::remove_file(&config_path).expect("Failed to remove config file");
                println!("{}", rust_i18n::t!("config_file_removed"));
            } else {
                println!("{}", rust_i18n::t!("operation_cancelled"));
            }
        } else {
            println!("{}", rust_i18n::t!("config_file_not_found"));
        }
    } else {
        // SGDK全体と設定を削除の前に確認
        println!("{}", rust_i18n::t!("uninstall_all_confirm"));

        use std::io::{self, Write};
        print!("> ");
        io::stdout().flush().unwrap();

        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();
        let input = input.trim().to_lowercase();

        if input != "y" && input != "yes" {
            println!("{}", rust_i18n::t!("operation_cancelled"));
            return;
        }

        // SGDK全体と設定を削除
        if config_path.exists() {
            // 設定からSGDKパスを取得
            let text = fs::read_to_string(&config_path).ok();
            if let Some(text) = text {
                if let Ok(mut doc) = text.parse::<Document>() {
                    if let Some(sgdk_path) = doc["sgdk"]["path"].as_str() {
                        let sgdk_dir = Path::new(sgdk_path);
                        if sgdk_dir.exists() {
                            println!(
                                "{}",
                                rust_i18n::t!(
                                    "removing_sgdk_installation",
                                    path = sgdk_dir.display()
                                )
                            );
                            fs::remove_dir_all(sgdk_dir).expect("Failed to remove SGDK directory");
                        }
                    }
                    // === ここから追加: エミュレータ(gens/blastem)も削除 ===
                    // config.tomlのパスを参照してgens/blastemディレクトリを削除
                    if let Some(gens_path) = doc
                        .get("emulator")
                        .and_then(|e| e.get("gens_path"))
                        .and_then(|v| v.as_str())
                    {
                        let gens_dir = std::path::Path::new(gens_path)
                            .parent()
                            .unwrap_or(std::path::Path::new(gens_path));
                        if gens_dir.exists() {
                            println!("Removing gens emulator: {}", gens_dir.display());
                            fs::remove_dir_all(gens_dir).expect("Failed to remove gens directory");
                        }
                    }
                    if let Some(blastem_path) = doc
                        .get("emulator")
                        .and_then(|e| e.get("blastem_path"))
                        .and_then(|v| v.as_str())
                    {
                        let blastem_dir = std::path::Path::new(blastem_path)
                            .parent()
                            .unwrap_or(std::path::Path::new(blastem_path));
                        if blastem_dir.exists() {
                            println!("Removing blastem emulator: {}", blastem_dir.display());
                            fs::remove_dir_all(blastem_dir)
                                .expect("Failed to remove blastem directory");
                        }
                    }
                    // config.tomlの[emulator]セクション削除
                    doc.remove("emulator");
                    fs::write(&config_path, doc.to_string()).expect("Failed to update config.toml");
                    // === ここまで追加 ===
                }
            }
        }

        // 設定ディレクトリ全体を削除
        if config_dir.exists() {
            fs::remove_dir_all(&config_dir).expect("Failed to remove config directory");
            println!("{}", rust_i18n::t!("sgdk_and_config_removed"));
        } else {
            println!("{}", rust_i18n::t!("nothing_to_remove"));
        }
    }
}

fn setup_emulator(emulator: &str, dir: Option<&str>) {
    let config_dir = config_dir()
        .expect("Unable to determine config directory")
        .join("sgdktool");

    let install_dir = if let Some(dir) = dir {
        PathBuf::from(dir)
    } else {
        config_dir.join(emulator)
    };

    if !install_dir.exists() {
        fs::create_dir_all(&install_dir).expect("Failed to create install directory");
    }

    // インストール処理
    match emulator {
        "gens" => setup_gens(&install_dir),
        "blastem" => setup_blastem(&install_dir),
        _ => {
            eprintln!(
                "Unsupported emulator: {}. Supported emulators: gens, blastem",
                emulator
            );
            std::process::exit(1);
        }
    }

    // 実行ファイルパスを探索してconfig.tomlに保存
    let exe_path = find_emulator_executable(&config_dir, emulator);
    if let Some(exe_path) = exe_path {
        let config_path = config_dir.join("config.toml");
        let mut doc = if config_path.exists() {
            fs::read_to_string(&config_path)
                .unwrap()
                .parse::<Document>()
                .unwrap()
        } else {
            Document::new()
        };
        doc["emulator"][format!("{}_path", emulator)] =
            value(exe_path.to_string_lossy().to_string());
        fs::write(&config_path, doc.to_string()).expect("Failed to write config.toml");
        println!("{} path saved to config.toml", emulator);
    }
}

fn setup_gens(install_dir: &Path) {
    println!("Setting up Gens KMod v0.7.3...");

    let url = "https://retrocdn.net/images/4/43/Gens_KMod_v0.7.3.7z";

    // Download the 7z file
    let response = get(url).expect("Failed to download Gens");
    let temp_file = NamedTempFile::new().expect("Failed to create temp file");
    let temp_path = temp_file.path().to_path_buf();

    // Write response to file
    let mut content = std::io::Cursor::new(response.bytes().expect("Failed to read response"));
    let mut file = fs::File::create(&temp_path).expect("Failed to create file");
    io_copy(&mut content, &mut file).expect("Failed to write to file");

    // Create the target directory if it doesn't exist
    if !install_dir.exists() {
        fs::create_dir_all(install_dir).expect("Failed to create install directory");
    }

    // Extract the 7z file
    println!("Extracting Gens KMod...");
    match sevenz_rust::decompress_file(&temp_path, install_dir) {
        Ok(_) => println!("Gens KMod v0.7.3 installed to {}", install_dir.display()),
        Err(e) => {
            eprintln!("Failed to extract Gens KMod: {}", e);
            std::process::exit(1);
        }
    }
}

fn setup_blastem(install_dir: &Path) {
    println!("Setting up BlastEm nightly build...");

    // Fetch the nightlies directory to find the latest build
    let base_url = "https://www.retrodev.com/blastem/nightlies/";
    let response = get(base_url).expect("Failed to connect to BlastEm nightlies page");
    let content = response.text().expect("Failed to read nightlies page");

    // Find the latest win64 nightly build
    // Look for links like "blastem-win64-0.6.3-pre-b42f00a3a937.zip"
    let re = regex::Regex::new(r"blastem-win64-[0-9\.]+.*?\.zip").unwrap();

    let latest_build = re
        .find_iter(&content)
        .next()
        .expect("Failed to find a win64 nightly build")
        .as_str();

    let url = format!("{}{}", base_url, latest_build);
    println!("Found latest build: {}", latest_build);

    // Download the zip file
    let response = get(&url).expect("Failed to download BlastEm");
    let mut temp_file = NamedTempFile::new().expect("Failed to create temp file");

    let mut content = std::io::Cursor::new(response.bytes().expect("Failed to read response"));
    io_copy(&mut content, &mut temp_file).expect("Failed to write to temp file");

    // Extract the zip file
    let file = fs::File::open(temp_file.path()).expect("Failed to open temp file");
    let mut archive = ZipArchive::new(file).expect("Failed to read zip archive");

    // Create the target directory if it doesn't exist
    if !install_dir.exists() {
        fs::create_dir_all(install_dir).expect("Failed to create install directory");
    }

    for i in 0..archive.len() {
        let mut file = archive
            .by_index(i)
            .expect("Failed to read file from archive");
        let outpath = install_dir.join(file.mangled_name());

        if file.name().ends_with('/') {
            fs::create_dir_all(&outpath).expect("Failed to create directory");
        } else {
            if let Some(p) = outpath.parent() {
                if !p.exists() {
                    fs::create_dir_all(p).expect("Failed to create parent directory");
                }
            }
            let mut outfile = fs::File::create(&outpath).expect("Failed to create output file");
            io_copy(&mut file, &mut outfile).expect("Failed to extract file");
        }
    }

    println!(
        "BlastEm nightly build installed to {}",
        install_dir.display()
    );
}

fn run_emulator(emulator: Option<&str>, rom_path: &str) {
    let config_dir = config_dir()
        .expect("Unable to determine config directory")
        .join("sgdktool");

    // Check if ROM file exists
    if !Path::new(rom_path).exists() {
        eprintln!("ROM file not found: {}", rom_path);
        std::process::exit(1);
    }

    let emulator_to_use = if let Some(emu) = emulator {
        emu.to_string()
    } else {
        // Auto-detect available emulator
        if find_emulator_executable(&config_dir, "gens").is_some() {
            "gens".to_string()
        } else if find_emulator_executable(&config_dir, "blastem").is_some() {
            "blastem".to_string()
        } else {
            eprintln!("No emulator found. Please run 'sgdktool setup-emu' first.");
            std::process::exit(1);
        }
    };

    let emulator_path = find_emulator_executable(&config_dir, &emulator_to_use);

    if let Some(exe_path) = emulator_path {
        run_with_wine(&exe_path, rom_path);
    } else {
        eprintln!(
            "Emulator '{}' not found. Please run 'sgdktool setup-emu {}' first.",
            emulator_to_use, emulator_to_use
        );
        std::process::exit(1);
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

fn run_with_wine(exe_path: &Path, rom_path: &str) {
    // Check if wine is available
    if which("wine").is_err() {
        eprintln!(
            "Wine is not installed or not in PATH. Please install wine to run Windows emulators."
        );
        std::process::exit(1);
    }

    println!("Running {} with wine...", exe_path.display());

    let absolute_rom_path =
        fs::canonicalize(rom_path).expect("Failed to get absolute path for ROM file");

    let status = Command::new("wine")
        .arg(exe_path)
        .arg(&absolute_rom_path)
        .status()
        .expect("Failed to run emulator with wine");

    if !status.success() {
        eprintln!("Emulator exited with error code: {:?}", status.code());
    }
}

fn show_help_output() {
    let exe = std::env::current_exe().unwrap_or_else(|_| "sgdktool".into());

    let status = Command::new(exe)
        .arg("help")
        .status()
        .expect(&rust_i18n::t!("help_failed"));

    if !status.success() {
        eprintln!("{}", rust_i18n::t!("help_warning"));
    }
}
