use std::fs;
use std::io;
use std::path::Path;

/// エクスポート用Webテンプレートのassetsディレクトリ
const WEB_TEMPLATE_DIR: &str = "assets/web-template";

/// web-exportコマンド本体
pub fn web_export(rom_path: Option<&str>, parent_dir: Option<&str>) {
    // ROMファイルのパス
    let rom_path = rom_path.unwrap_or("out/rom.bin");
    let rom_src = Path::new(rom_path);
    if !rom_src.exists() {
        eprintln!("❌ ROM file not found: {}", rom_src.display());
        std::process::exit(1);
    }

    // 出力先ディレクトリ（<dir>/web-export）
    let parent_dir = parent_dir.unwrap_or(".");
    let out_path = Path::new(parent_dir).join("web-export");

    // 出力先ディレクトリ作成
    if !out_path.exists() {
        fs::create_dir_all(&out_path).expect("Failed to create output directory");
    }

    // テンプレート一式をコピー
    let template_dir = Path::new(WEB_TEMPLATE_DIR);
    if !template_dir.exists() {
        eprintln!(
            "❌ Web template directory not found: {}",
            template_dir.display()
        );
        std::process::exit(1);
    }
    copy_dir_all(template_dir, &out_path).expect("Failed to copy web template files");

    // ROMファイルをコピー
    let rom_dest = out_path.join("rom.bin");
    fs::copy(&rom_src, &rom_dest).expect("Failed to copy ROM file");

    println!("✅ Web export complete!");
    println!("  Output directory: {}", out_path.display());
    println!("  Open {}/index.html in your browser.", out_path.display());
}

/// ディレクトリを再帰的にコピー（fs_extraを使わず標準で）
fn copy_dir_all(src: &Path, dst: &Path) -> io::Result<()> {
    if !dst.exists() {
        fs::create_dir_all(dst)?;
    }
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let file_type = entry.file_type()?;
        let dest_path = dst.join(entry.file_name());
        if file_type.is_dir() {
            copy_dir_all(&entry.path(), &dest_path)?;
        } else {
            fs::copy(entry.path(), dest_path)?;
        }
    }
    Ok(())
}
