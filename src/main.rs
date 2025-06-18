use clap::{Parser, Subcommand};
use dirs::config_dir;
use fs_extra::dir::{CopyOptions, copy};
use std::fs;
use std::os::unix::fs::symlink;
use std::path::{Path, PathBuf};
use std::process::Command;
use toml_edit::{Document, value};
use which::which;

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
            Commands::New { name } => {
                create_project(&name);
            }
            Commands::Make { project, extra } => {
                build_project(&project, extra);
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
                if let Ok(doc) = text.parse::<Document>() {
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
