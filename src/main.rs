use clap::{Parser, Subcommand};

mod commands;
use commands::doc::show_sgdk_doc_status;
use commands::doctor::run_doctor_and_info;
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

    /// Show SGDK documentation status
    Doc,

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
    Uninstall,
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
            Commands::Uninstall => {
                uninstall_sgdk();
            }
            Commands::Doc => {
                show_sgdk_doc_status();
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
            Command::new("doc")
                .about(if is_japanese {
                    "SGDKドキュメントの生成状況を表示"
                } else {
                    "Show SGDK documentation status"
                }),
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
        Some(("uninstall", _sub_matches)) => Cli {
            command: Some(Commands::Uninstall),
        },
        Some(("doc", _sub_matches)) => Cli {
            command: Some(Commands::Doc),
        },
        _ => Cli { command: None },
    }
}
