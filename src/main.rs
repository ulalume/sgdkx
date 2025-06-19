use clap::{Parser, Subcommand};
use dirs::config_dir;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use toml_edit::DocumentMut;

mod commands;

use commands::make::build_project;
use commands::new::create_project;
use commands::run::run_emulator;
use commands::setup::setup_sgdk;
use commands::setup_emu::setup_emulator;
use commands::uninstall::uninstall_sgdk;

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

        /// Branch, tag, or commit ID to clone (defaults to master)
        #[arg(long, default_value = "master")]
        version: String,
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
        /// Additional options to pass to make
        #[arg(last = true)]
        extra: Vec<String>,
    },

    /// Run ROM file with emulator
    Run {
        /// Emulator to use (gens or blastem, defaults to available emulator)
        #[arg(long)]
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
            Commands::Setup { dir, version } => {
                setup_sgdk(dir.as_deref(), &version);
            }
            Commands::SetupEmu { emulator, dir } => {
                setup_emulator(&emulator, dir.as_deref());
            }
            Commands::New { name } => {
                create_project(&name);
            }
            Commands::Make { extra } => {
                build_project(&extra);
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
                    clap::Arg::new("version")
                        .long("version")
                        .default_value("master")
                        .help(if is_japanese {
                            "クローンするブランチ名・タグ・コミットID（省略時はmaster）"
                        } else {
                            "Branch, tag, or commit ID to clone (defaults to master)"
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
                .arg(
                    clap::Arg::new("emulator")
                        .long("emulator")
                        .help(if is_japanese {
                            "使用するエミュレータ (gens または blastem、省略時は利用可能なエミュレータ)"
                        } else {
                            "Emulator to use (gens or blastem, defaults to available emulator)"
                        }),
                )
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
                version: sub_matches.get_one::<String>("version").unwrap().clone(),
            }),
        },
        Some(("new", sub_matches)) => Cli {
            command: Some(Commands::New {
                name: sub_matches.get_one::<String>("name").unwrap().clone(),
            }),
        },
        Some(("make", sub_matches)) => Cli {
            command: Some(Commands::Make {
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

fn run_doctor_and_info() {
    show_help_output();

    println!("\n{}", rust_i18n::t!("environment_check"));

    for tool in ["git", "make", "java", "compiledb"].iter() {
        check_tool(tool);
    }

    #[cfg(not(target_os = "windows"))]
    check_tool("wine");

    let config_path = config_dir().unwrap().join("sgdktool").join("config.toml");

    println!("\n{}", rust_i18n::t!("sgdk_config_info"));

    if config_path.exists() {
        let text = fs::read_to_string(&config_path).unwrap();
        let doc = text.parse::<DocumentMut>().unwrap();
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
