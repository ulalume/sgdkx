use crate::path;
use clap::Parser;
use std::fs;
use std::io::IsTerminal;

#[derive(Parser)]
pub struct Args {
    /// Skip the confirmation prompt (required when non-interactive)
    #[arg(short = 'y', long = "yes")]
    yes: bool,
}

pub fn run(args: &Args) {
    let config_dir = path::config_dir();

    // Destructive: confirm unless --yes. On a terminal, prompt y/N; non-interactively, require
    // --yes rather than reading stdin (which would hang or read EOF in a pipeline/CI).
    if !args.yes {
        if !std::io::stdin().is_terminal() {
            eprintln!("❌ refusing to uninstall non-interactively. Re-run with --yes to confirm.");
            std::process::exit(1);
        }
        use std::io::{self, Write};
        println!("⚠️  Completely remove the SGDK environment and config?");
        print!("[y/N]> ");
        io::stdout().flush().unwrap();
        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();
        let input = input.trim().to_lowercase();
        if input != "y" && input != "yes" {
            println!("❌ Operation cancelled.");
            return;
        }
    }

    // Everything (SGDK, toolchain, JRE, gdb, BlastEm, config.toml) lives under config_dir,
    // so a single recursive remove cleans it all — no need to read config.toml or remove
    // components individually.
    if config_dir.exists() {
        println!("🗑️  Removing {}", config_dir.display());
        fs::remove_dir_all(&config_dir).expect("Failed to remove config directory");
        println!("✅ SGDK environment and configuration removed");
    } else {
        println!("⚠️  Nothing to remove (no install found at {})", config_dir.display());
    }
}
