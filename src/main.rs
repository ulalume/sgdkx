use clap::{Parser, Subcommand};

mod commands;
mod path;
mod release;
use commands::blastem;
use commands::doc;
use commands::doctor;
use commands::gdb;
use commands::make;
use commands::new;
use commands::open;
use commands::setup;
use commands::setup_emu;
use commands::setup_web;
use commands::uninstall;
use commands::web_export;
use commands::web_server;

/// Unofficial tools for SGDK workflow
#[derive(Parser)]
#[command(name = "sgdkx")]
#[command(version = env!("CARGO_PKG_VERSION"))]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Setup SGDK for development
    Setup(setup::Args),

    /// Setup emulator for running ROM files
    SetupEmu(setup_emu::Args),

    /// Create a new SGDK project
    New(new::Args),

    /// Build the project: thin wrapper around make (args passed straight through)
    Make(make::Args),

    /// Run the bundled BlastEm (args passed straight through, e.g. out/rom.bin)
    Blastem(blastem::Args),

    /// Run m68k-elf-gdb (args passed straight through, e.g. out/rom.out)
    Gdb(gdb::Args),

    /// Setup web export template
    SetupWeb(setup_web::Args),

    /// Export ROM and web emulator template for web deployment
    WebExport(web_export::Args),

    /// Serve web-export directory with HTTP server (with COOP/COEP headers)
    WebServer(web_server::Args),

    /// Show SGDK documentation status
    Doc,

    /// Open SGDK installation directory
    Open(open::Args),

    /// Uninstall SGDK installation and configuration
    Uninstall,
}

fn main() {
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
            Commands::Make(args) => {
                make::run(args);
            }
            Commands::Blastem(args) => {
                blastem::run(args);
            }
            Commands::Gdb(args) => {
                gdb::run(args);
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
