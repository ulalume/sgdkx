use crate::path;
use clap::Parser;
use std::fs;
use std::path::{Path, PathBuf};
use toml_edit::DocumentMut;

// Import constants from setup_web
//
#[derive(Parser)]
pub struct Args {
    /// ROM file path (defaults to out/rom.bin)
    #[arg(long, default_value = "out/rom.bin")]
    rom: String,
    /// Parent directory to create web-export in (defaults to current directory)
    #[arg(long, default_value = ".")]
    dir: String,
}

/// web-exportコマンド本体
pub fn run(args: &Args) {
    // ROMファイルのパス
    let rom_path = &args.rom;
    let rom_src = Path::new(rom_path);
    if !rom_src.exists() {
        eprintln!("❌ ROM file not found: {}", rom_src.display());
        std::process::exit(1);
    }

    // 出力先ディレクトリ（<dir>/web-export）
    let parent_dir = &args.dir;
    let out_path = Path::new(parent_dir).join("web-export");

    // 出力先ディレクトリ作成
    if !out_path.exists() {
        fs::create_dir_all(&out_path).expect("Failed to create output directory");
    }

    // テンプレートの場所を確認
    let template_dir = get_template_directory();

    // テンプレートディレクトリが存在しない場合、setup-webを促す
    match template_dir {
        Some(template_dir) => {
            println!("Using web template from: {}", template_dir.display());
            copy_directory(&template_dir, &out_path).expect("Failed to copy template files");
        }
        None => {
            eprintln!("❌ Web template not found. Please run `sgdkx setup-web` first.");
            std::process::exit(1);
        }
    }

    // ROMファイルをコピー
    let rom_dest = out_path.join("rom.bin");
    fs::copy(&rom_src, &rom_dest).expect("Failed to copy ROM file");

    println!("✅ Web export complete!");
    println!("  Output directory: {}", out_path.display());
    println!("  Run sgdkx web-server");
}

// テンプレートディレクトリを取得する関数
fn get_template_directory() -> Option<PathBuf> {
    let config_path = path::config_dir().join("config.toml");

    if config_path.exists() {
        let config_str = match fs::read_to_string(&config_path) {
            Ok(s) => s,
            Err(_) => return None,
        };

        let doc = match config_str.parse::<DocumentMut>() {
            Ok(d) => d,
            Err(_) => return None,
        };

        // config.tomlからテンプレートパスを取得
        if let Some(web_export) = doc.get("web_export") {
            if let Some(path_value) = web_export.get("template_path").and_then(|v| v.as_str()) {
                let template_dir = PathBuf::from(path_value);
                if template_dir.exists() {
                    return Some(template_dir);
                }
            }
        }
    }

    None
}

// ディレクトリ全体をコピーする関数
fn copy_directory(src: &Path, dst: &Path) -> std::io::Result<()> {
    if !dst.exists() {
        fs::create_dir_all(dst)?;
    }

    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let src_path = entry.path();
        let dst_path = dst.join(entry.file_name());

        if src_path.is_dir() {
            copy_directory(&src_path, &dst_path)?;
        } else {
            fs::copy(&src_path, &dst_path)?;
        }
    }

    Ok(())
}
