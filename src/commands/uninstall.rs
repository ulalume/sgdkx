use crate::path;
use clap::Parser;
use std::fs;
use std::io::IsTerminal;
use std::path::Path;
use toml_edit::DocumentMut;

#[derive(Parser)]
pub struct Args {
    /// Skip the confirmation prompt (required when non-interactive)
    #[arg(short = 'y', long = "yes")]
    yes: bool,
}

pub fn run(args: &Args) {
    let config_dir = path::config_dir();

    let config_path = config_dir.join("config.toml");

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

    // SGDK全体と設定を削除
    if config_path.exists() {
        // 設定からSGDKパスを取得
        let text = fs::read_to_string(&config_path).ok();
        if let Some(text) = text
            && let Ok(mut doc) = text.parse::<DocumentMut>() {
                let (sgdk_path_opt, _) = crate::commands::new::get_sgdk_config(&doc);
                if let Some(sgdk_path) = sgdk_path_opt {
                    let sgdk_dir = Path::new(sgdk_path);
                    if sgdk_dir.exists() {
                        println!(
                            "🗑️  Removing SGDK installation: {}",
                            sgdk_dir.display()
                        );
                        fs::remove_dir_all(sgdk_dir).expect("Failed to remove SGDK directory");
                    }
                }
                // also remove the BlastEm emulator directory
                if let Some(blastem_path) = doc
                    .get("emulator")
                    .and_then(|e| e.get("blastem_path"))
                    .and_then(|v| v.as_str())
                {
                    let blastem_dir = std::path::Path::new(blastem_path)
                        .parent()
                        .unwrap_or(std::path::Path::new(blastem_path));
                    if blastem_dir.exists() {
                        println!("Removing blastem emulator: {}", blastem_dir.display());
                        fs::remove_dir_all(blastem_dir)
                            .expect("Failed to remove blastem directory");
                    }
                }
                // config.tomlの[emulator]セクション削除
                doc.remove("emulator");
                fs::write(&config_path, doc.to_string()).expect("Failed to update config.toml");
                // === ここまで追加 ===
            }
    }

    // 設定ディレクトリ全体を削除
    if config_dir.exists() {
        fs::remove_dir_all(&config_dir).expect("Failed to remove config directory");
        println!("✅ SGDK and configuration completely removed");
    } else {
        println!("⚠️  Nothing to remove found");
    }
}
