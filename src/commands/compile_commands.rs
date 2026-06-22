use crate::commands::new::generate_compile_commands;
use crate::path;
use clap::Parser;
use std::path::Path;

#[derive(Parser)]
pub struct Args {
    /// Project directory to (re)generate compile_commands.json in (defaults to the current dir)
    #[arg(short = 'p', long = "path", default_value = ".")]
    path: String,
}

/// Regenerate `compile_commands.json` for a project (e.g. after adding/removing source files).
/// Same generator `new` runs at creation time: parses a `make -nwB` dry-run, no external tools.
pub fn run(args: &Args) {
    if !path::is_installed() {
        eprintln!("❌ SGDK not installed. Please run `sgdkx install` first.");
        std::process::exit(1);
    }
    let project = Path::new(&args.path);
    if !project.is_dir() {
        eprintln!("❌ not a directory: {}", project.display());
        std::process::exit(1);
    }
    if !project.join("Makefile").exists() {
        eprintln!(
            "❌ no Makefile in {} — run this inside an sgdkx project (see `sgdkx new`).",
            project.display()
        );
        std::process::exit(1);
    }
    generate_compile_commands(project);
}
