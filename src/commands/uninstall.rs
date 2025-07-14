use crate::path;
use rust_i18n;
use std::fs;
use std::path::Path;
use toml_edit::DocumentMut;

pub fn run() {
    let config_dir = path::config_dir();

    let config_path = config_dir.join("config.toml");

    // SGDK全体と設定を削除の前に確認
    println!("{}", rust_i18n::t!("uninstall_all_confirm"));

    use std::io::{self, Write};
    print!("> ");
    io::stdout().flush().unwrap();

    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap();
    let input = input.trim().to_lowercase();

    if input != "y" && input != "yes" {
        println!("{}", rust_i18n::t!("operation_cancelled"));
        return;
    }

    // SGDK全体と設定を削除
    if config_path.exists() {
        // 設定からSGDKパスを取得
        let text = fs::read_to_string(&config_path).ok();
        if let Some(text) = text {
            if let Ok(mut doc) = text.parse::<DocumentMut>() {
                let (sgdk_path_opt, _) = crate::commands::new::get_sgdk_config(&doc);
                if let Some(sgdk_path) = sgdk_path_opt {
                    let sgdk_dir = Path::new(sgdk_path);
                    if sgdk_dir.exists() {
                        println!(
                            "{}",
                            rust_i18n::t!("removing_sgdk_installation", path = sgdk_dir.display())
                        );
                        fs::remove_dir_all(sgdk_dir).expect("Failed to remove SGDK directory");
                    }
                }
                // === ここから追加: エミュレータ(gens/blastem)も削除 ===
                // config.tomlのパスを参照してgens/blastemディレクトリを削除
                if let Some(gens_path) = doc
                    .get("emulator")
                    .and_then(|e| e.get("gens_path"))
                    .and_then(|v| v.as_str())
                {
                    let gens_dir = std::path::Path::new(gens_path)
                        .parent()
                        .unwrap_or(std::path::Path::new(gens_path));
                    if gens_dir.exists() {
                        println!("Removing gens emulator: {}", gens_dir.display());
                        fs::remove_dir_all(gens_dir).expect("Failed to remove gens directory");
                    }
                }
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
    }

    // 設定ディレクトリ全体を削除
    if config_dir.exists() {
        fs::remove_dir_all(&config_dir).expect("Failed to remove config directory");
        println!("{}", rust_i18n::t!("sgdk_and_config_removed"));
    } else {
        println!("{}", rust_i18n::t!("nothing_to_remove"));
    }
}
