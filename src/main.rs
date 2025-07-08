use clap::{Parser, Subcommand};

mod commands;
use commands::doc;
use commands::doctor;
use commands::make;
use commands::new;
use commands::open;
use commands::run;
use commands::setup;
use commands::setup_emu;
use commands::setup_web;
use commands::uninstall;
use commands::web_export;
use commands::web_server;

// 多言語化の初期化
rust_i18n::i18n!("locales");

/// A CLI tool for SGDK-based development
#[derive(Parser)]
#[command(name = "sgdktool")]
#[command(version = "0.1.1")]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Setup SGDK for development
    Setup(setup::Args),

    /// Show SGDK documentation status
    Doc,

    /// Setup emulator for running ROM files
    SetupEmu(setup_emu::Args),

    /// Create a new SGDK project
    New(new::Args),

    /// Build project using make
    Make(make::Args),

    /// Run ROM file with emulator
    Run(run::Args),

    /// Uninstall SGDK installation and configuration
    Uninstall,

    /// Export ROM and web emulator template for web deployment
    WebExport(web_export::Args),

    /// Serve web-export directory with HTTP server (with COOP/COEP headers)
    WebServer(web_server::Args),

    /// Open SGDK installation directory
    Open(open::Args),

    /// Setup web export template
    SetupWeb(setup_web::Args),
}

fn main() {
    // ロケールを設定
    init_locale();

    // 多言語化対応のCLI作成
    let cli = create_localized_cli();

    match &cli.command {
        Some(cmd) => match cmd {
            Commands::Setup(args) => {
                setup::run(&args);
            }
            Commands::SetupEmu(args) => {
                setup_emu::run(&args);
            }
            Commands::New(args) => {
                new::run(&args);
            }
            Commands::Make(args) => {
                make::run(&args);
            }
            Commands::Run(args) => {
                run::run(args);
            }
            Commands::Uninstall => {
                uninstall::run();
            }
            Commands::Doc => {
                doc::run();
            }
            Commands::WebExport(args) => {
                web_export::run(args);
            }
            Commands::WebServer(args) => {
                web_server::run(args);
            }
            Commands::Open(args) => {
                open::run(args);
            }
            Commands::SetupWeb(args) => {
                setup_web::run(args);
            }
        },
        None => {
            // コマンドが指定されなかったときに実行したいロジック
            doctor::run();
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
            Command::new("open")
                .about(if is_japanese {
                    "SGDKインストールディレクトリを開く"
                } else {
                    "Open SGDK installation directory"
                }),
        )
        .subcommand(
            Command::new("setup-web")
                .about(if is_japanese {
                    "Webエクスポートテンプレートをセットアップ"
                } else {
                    "Setup web export template"
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
                    "SGDKサンプルから新しいプロジェクトを作成"
                } else {
                    "Create new project from SGDK sample"
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
        )
        .subcommand(
            Command::new("web-export")
                .about(if is_japanese {
                    "ROMとWebエミュレータテンプレートをエクスポート"
                } else {
                    "Export ROM and web emulator template for web deployment"
                })
                .arg(
                    clap::Arg::new("rom")
                        .long("rom")
                        .default_value("out/rom.bin")
                        .help(if is_japanese {
                            "ROMファイルのパス（省略時は out/rom.bin）"
                        } else {
                            "ROM file path (defaults to out/rom.bin)"
                        }),
                )
                .arg(
                    clap::Arg::new("dir")
                        .long("dir")
                        .default_value(".")
                        .help(if is_japanese {
                            "web-exportディレクトリを作成する親ディレクトリ（省略時はカレントディレクトリ）"
                        } else {
                            "Parent directory to create web-export in (defaults to current directory)"
                        }),
                ),
        )
        .subcommand(
            Command::new("web-server")
                .about(if is_japanese {
                    "web-exportディレクトリをHTTPサーバで公開（COOP/COEPヘッダ付き）"
                } else {
                    "Serve web-export directory with HTTP server (with COOP/COEP headers)"
                })
                .arg(
                    clap::Arg::new("dir")
                        .long("dir")
                        .default_value("web-export")
                        .help(if is_japanese {
                            "公開するディレクトリ（省略時はweb-export）"
                        } else {
                            "Directory to serve (defaults to web-export)"
                        }),
                )
                .arg(
                    clap::Arg::new("port")
                        .long("port")
                        .default_value("8080")
                        .value_parser(clap::value_parser!(u16))
                        .help(if is_japanese {
                            "待ち受けポート番号（省略時は8080）"
                        } else {
                            "Port to listen on (defaults to 8080)"
                        }),
                )

        );

    let matches = app.get_matches();

    // マッチした結果をCli構造体に変換
    match matches.subcommand() {
        Some(("setup", sub_matches)) => Cli {
            command: Some(Commands::Setup(setup::Args::new(
                sub_matches.get_one::<String>("version").unwrap().clone(),
            ))),
        },
        Some(("new", sub_matches)) => Cli {
            command: Some(Commands::New(new::Args::new(
                sub_matches.get_one::<String>("name").unwrap().clone(),
            ))),
        },
        Some(("make", sub_matches)) => Cli {
            command: Some(Commands::Make(make::Args::new(
                sub_matches
                    .get_many::<String>("extra")
                    .unwrap_or_default()
                    .map(|s| s.clone())
                    .collect(),
            ))),
        },
        Some(("setup-emu", sub_matches)) => Cli {
            command: Some(Commands::SetupEmu(setup_emu::Args::new(
                sub_matches.get_one::<String>("emulator").unwrap().clone(),
            ))),
        },
        Some(("run", sub_matches)) => Cli {
            command: Some(Commands::Run(run::Args::new(
                sub_matches.get_one::<String>("emulator").cloned(),
                sub_matches.get_one::<String>("rom").cloned(),
            ))),
        },
        Some(("uninstall", _sub_matches)) => Cli {
            command: Some(Commands::Uninstall),
        },
        Some(("doc", _sub_matches)) => Cli {
            command: Some(Commands::Doc),
        },
        Some(("web-export", sub_matches)) => Cli {
            command: Some(Commands::WebExport(web_export::Args::new(
                sub_matches.get_one::<String>("rom").cloned(),
                sub_matches.get_one::<String>("dir").cloned(),
            ))),
        },
        Some(("web-server", sub_matches)) => Cli {
            command: Some(Commands::WebServer(web_server::Args::new(
                sub_matches.get_one::<String>("dir").cloned(),
                sub_matches.get_one::<u16>("port").cloned(),
            ))),
        },
        Some(("open", _sub_matches)) => Cli {
            command: Some(Commands::Open(open::Args {})),
        },
        Some(("setup-web", _sub_matches)) => Cli {
            command: Some(Commands::SetupWeb(setup_web::Args {})),
        },
        _ => Cli { command: None },
    }
}
