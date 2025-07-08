use clap::Parser;
use std::process::Command;

#[derive(Parser)]
pub struct Args {}

pub fn run(_args: &Args) {
    let target_dir = dirs::config_dir().unwrap().join("sgdktool");

    if !target_dir.exists() {
        println!("SGDK directory does not exist: {}", target_dir.display());
        println!("Please run `sgdktool setup` first.");
        return;
    }

    println!("Opening SGDK directory: {}", target_dir.display());

    let result = if cfg!(target_os = "macos") {
        Command::new("open").arg(&target_dir).spawn()
    } else if cfg!(target_os = "windows") {
        Command::new("explorer").arg(&target_dir).spawn()
    } else {
        // Assume Linux or other Unix-like OS for xdg-open
        Command::new("xdg-open").arg(&target_dir).spawn()
    };

    match result {
        Ok(_) => println!("Successfully opened directory."),
        Err(e) => eprintln!("Failed to open directory: {}", e),
    }
}
