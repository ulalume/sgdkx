use std::path::{Path, PathBuf};
use std::fs;
use dirs::config_dir;
use toml_edit::{Document, value};
use rust_i18n;
use reqwest::blocking::get;
use tempfile::NamedTempFile;
use std::io::copy as io_copy;
use zip::ZipArchive;
use sevenz_rust;

pub fn setup_emulator(emulator: &str, dir: Option<&str>) {
    let config_dir = config_dir()
        .expect("Unable to determine config directory")
        .join("sgdktool");

    let install_dir = if let Some(dir) = dir {
        PathBuf::from(dir)
    } else {
        config_dir.join(emulator)
    };

    if !install_dir.exists() {
        fs::create_dir_all(&install_dir).expect("Failed to create install directory");
    }

    // インストール処理
    match emulator {
        "gens" => setup_gens(&install_dir),
        "blastem" => setup_blastem(&install_dir),
        _ => {
            eprintln!(
                "Unsupported emulator: {}. Supported emulators: gens, blastem",
                emulator
            );
            std::process::exit(1);
        }
    }

    // 実行ファイルパスを探索してconfig.tomlに保存
    let exe_path = crate::commands::run::find_emulator_executable(&config_dir, emulator);
    if let Some(exe_path) = exe_path {
        let config_path = config_dir.join("config.toml");
        let mut doc = if config_path.exists() {
            fs::read_to_string(&config_path)
                .unwrap()
                .parse::<Document>()
                .unwrap()
        } else {
            Document::new()
        };
        doc["emulator"][format!("{}_path", emulator)] =
            value(exe_path.to_string_lossy().to_string());
        fs::write(&config_path, doc.to_string()).expect("Failed to write config.toml");
        println!("{} path saved to config.toml", emulator);
    }
}

fn setup_gens(install_dir: &Path) {
    println!("Setting up Gens KMod v0.7.3...");

    let url = "https://retrocdn.net/images/4/43/Gens_KMod_v0.7.3.7z";

    // Download the 7z file
    let response = get(url).expect("Failed to download Gens");
    let temp_file = NamedTempFile::new().expect("Failed to create temp file");
    let temp_path = temp_file.path().to_path_buf();

    // Write response to file
    let mut content = std::io::Cursor::new(response.bytes().expect("Failed to read response"));
    let mut file = fs::File::create(&temp_path).expect("Failed to create file");
    io_copy(&mut content, &mut file).expect("Failed to write to file");

    // Create the target directory if it doesn't exist
    if !install_dir.exists() {
        fs::create_dir_all(install_dir).expect("Failed to create install directory");
    }

    // Extract the 7z file
    println!("Extracting Gens KMod...");
    match sevenz_rust::decompress_file(&temp_path, install_dir) {
        Ok(_) => println!("Gens KMod v0.7.3 installed to {}", install_dir.display()),
        Err(e) => {
            eprintln!("Failed to extract Gens KMod: {}", e);
            std::process::exit(1);
        }
    }
}

fn setup_blastem(install_dir: &Path) {
    println!("Setting up BlastEm nightly build...");

    // Fetch the nightlies directory to find the latest build
    let base_url = "https://www.retrodev.com/blastem/nightlies/";
    let response = get(base_url).expect("Failed to connect to BlastEm nightlies page");
    let content = response.text().expect("Failed to read nightlies page");

    // Find the latest win64 nightly build
    // Look for links like "blastem-win64-0.6.3-pre-b42f00a3a937.zip"
    let re = regex::Regex::new(r"blastem-win64-[0-9\.]+.*?\.zip").unwrap();

    let latest_build = re
        .find_iter(&content)
        .next()
        .expect("Failed to find a win64 nightly build")
        .as_str();

    let url = format!("{}{}", base_url, latest_build);
    println!("Found latest build: {}", latest_build);

    // Download the zip file
    let response = get(&url).expect("Failed to download BlastEm");
    let mut temp_file = NamedTempFile::new().expect("Failed to create temp file");

    let mut content = std::io::Cursor::new(response.bytes().expect("Failed to read response"));
    io_copy(&mut content, &mut temp_file).expect("Failed to write to temp file");

    // Extract the zip file
    let file = fs::File::open(temp_file.path()).expect("Failed to open temp file");
    let mut archive = ZipArchive::new(file).expect("Failed to read zip archive");

    // Create the target directory if it doesn't exist
    if !install_dir.exists() {
        fs::create_dir_all(install_dir).expect("Failed to create install directory");
    }

    for i in 0..archive.len() {
        let mut file = archive
            .by_index(i)
            .expect("Failed to read file from archive");
        let outpath = install_dir.join(file.mangled_name());

        if file.name().ends_with('/') {
            fs::create_dir_all(&outpath).expect("Failed to create directory");
        } else {
            if let Some(p) = outpath.parent() {
                if !p.exists() {
                    fs::create_dir_all(p).expect("Failed to create parent directory");
                }
            }
            let mut outfile = fs::File::create(&outpath).expect("Failed to create output file");
            io_copy(&mut file, &mut outfile).expect("Failed to extract file");
        }
    }

    println!(
        "BlastEm nightly build installed to {}",
        install_dir.display()
    );
} 