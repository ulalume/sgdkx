use rust_embed::RustEmbed;
use std::fs;
use std::path::Path;

/// RustEmbedでassets/web-templateを埋め込む
#[derive(RustEmbed)]
#[folder = "assets/web-template/"]
struct WebTemplate;

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

    // 埋め込んだテンプレートファイルを展開
    for file in WebTemplate::iter() {
        let file_str = file.as_ref();
        // .d.tsファイルは除外
        if file_str.ends_with(".d.ts") {
            continue;
        }
        let data = WebTemplate::get(file_str).unwrap();
        let dest_path = out_path.join(file_str);
        if let Some(parent) = dest_path.parent() {
            if !parent.exists() {
                fs::create_dir_all(parent).expect("Failed to create template subdirectory");
            }
        }
        fs::write(&dest_path, data.data).expect("Failed to write template file");
    }

    // ROMファイルをコピー
    let rom_dest = out_path.join("rom.bin");
    fs::copy(&rom_src, &rom_dest).expect("Failed to copy ROM file");

    println!("✅ Web export complete!");
    println!("  Output directory: {}", out_path.display());
    println!("  Open {}/index.html in your browser.", out_path.display());
}
