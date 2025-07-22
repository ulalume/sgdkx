use clap::{Parser, Subcommand};

mod commands;
mod path;
use commands::doc;
use commands::doctor;
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

/// Unofficial tools for SGDK workflow
#[derive(Parser)]
#[command(name = "sgdkx")]
#[command(version = "0.1.3")]
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
    init_locale();
    let cli = Cli::parse();

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
            // Run doctor command when no subcommand is specified
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
