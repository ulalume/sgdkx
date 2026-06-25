use clap::{Parser, Subcommand};

mod commands;
mod path;
mod release;
use commands::blastem;
use commands::compile_commands;
use commands::doc;
use commands::doctor;
use commands::gdb;
use commands::install;
use commands::make;
use commands::new;
use commands::open;
use commands::uninstall;

/// One-command native SGDK dev environment. Unofficial, cross-platform CLI.
#[derive(Parser)]
#[command(name = "sgdkx")]
#[command(version = env!("CARGO_PKG_VERSION"))]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Install/update the self-contained SGDK environment (SGDK, toolchain, JRE, gdb, BlastEm)
    Install(install::Args),

    /// Create a new SGDK project
    New(new::Args),

    /// Build the project: thin wrapper around make (args passed straight through)
    Make(make::Args),

    /// Run the bundled BlastEm (args passed straight through, e.g. out/rom.bin)
    Blastem(blastem::Args),

    /// Run m68k-elf-gdb (args passed straight through, e.g. out/rom.out)
    Gdb(gdb::Args),

    /// Regenerate compile_commands.json (e.g. after adding/removing source files)
    #[allow(clippy::enum_variant_names)] // name must stay for the `compile-commands` command
    CompileCommands(compile_commands::Args),

    /// Show SGDK documentation status
    Doc,

    /// Open the SGDK installation directory
    Open(open::Args),

    /// Uninstall the SGDK environment and configuration
    Uninstall(uninstall::Args),
}

fn main() {
    let cli = Cli::parse();

    match &cli.command {
        Some(cmd) => match cmd {
            Commands::Install(args) => install::run(args),
            Commands::New(args) => new::run(args),
            Commands::Make(args) => make::run(args),
            Commands::Blastem(args) => blastem::run(args),
            Commands::Gdb(args) => gdb::run(args),
            Commands::CompileCommands(args) => compile_commands::run(args),
            Commands::Doc => doc::run(),
            Commands::Open(args) => open::run(args),
            Commands::Uninstall(args) => uninstall::run(args),
        },
        None => {
            // No subcommand: print help (via clap, no subprocess) then the doctor check.
            use clap::CommandFactory;
            let _ = Cli::command().print_help();
            println!();
            doctor::run();
        }
    }
}
