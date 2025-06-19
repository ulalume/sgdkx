use dirs::config_dir;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use toml_edit::{DocumentMut, value};
use which::which;

// 多言語化
use rust_i18n;

pub fn setup_sgdk(dir: Option<&str>, version: &str) {
    if which("git").is_err() {
        eprintln!("{}", rust_i18n::t!("git_not_found"));
        std::process::exit(1);
    }

    let target_dir = if let Some(custom_dir) = dir {
        PathBuf::from(custom_dir)
    } else {
        config_dir()
            .expect("Failed to get config directory")
            .join("sgdktool")
            .join("SGDK")
    };

    if dir.is_none() {
        if rust_i18n::locale().to_string() == "ja" {
            println!(
                "📁 デフォルト設定ディレクトリを使用: {}",
                target_dir.display()
            );
        } else {
            println!(
                "📁 Using default config directory: {}",
                target_dir.display()
            );
        }
    }
    if target_dir.exists() {
        println!("{}", rust_i18n::t!("sgdk_exists_overwrite"));
        use std::io::{self, Write};
        print!("{}", rust_i18n::t!("prompt"));
        io::stdout().flush().unwrap();

        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();
        let input = input.trim().to_lowercase();

        if input != "y" && input != "" {
            println!("{}", rust_i18n::t!("operation_cancelled"));
            std::process::exit(0);
        }

        println!("{}", rust_i18n::t!("saving_config"));
        let config_dir = config_dir()
            .expect("Failed to get config directory")
            .join("sgdktool");
        fs::create_dir_all(&config_dir).expect("Failed to create config directory");
        let config_path = config_dir.join("config.toml");

        let mut doc = if config_path.exists() {
            let text =
                fs::read_to_string(&config_path).expect(&rust_i18n::t!("config_read_failed"));
            text.parse::<DocumentMut>()
                .expect(&rust_i18n::t!("toml_parse_failed"))
        } else {
            DocumentMut::new()
        };
        let abs_path = target_dir
            .canonicalize()
            .expect("Failed to get absolute path");
        doc["sgdk"]["path"] = value(abs_path.to_str().unwrap());
        doc["sgdk"]["version"] = value(version);

        fs::write(&config_path, doc.to_string()).expect("Failed to write config.toml");
        println!("{}", rust_i18n::t!("config_only_created"));
        return;
    }

    println!("{}", rust_i18n::t!("cloning_sgdk"));
    if let Some(parent) = target_dir.parent() {
        fs::create_dir_all(parent).expect("Failed to create parent directory");
    }

    // 判定: versionがSHA-1形式(7-40桁の16進数)ならcommit IDとみなす
    let is_commit_id = {
        let v = version;
        let len = v.len();
        len >= 7 && len <= 40 && v.chars().all(|c| c.is_ascii_hexdigit())
    };

    let clone_status = if is_commit_id {
        // commit IDの場合はmasterをcloneしてcheckout
        Command::new("git")
            .args([
                "clone",
                "https://github.com/Stephane-D/SGDK",
                target_dir.to_str().unwrap(),
            ])
            .status()
            .expect("git clone failed")
    } else {
        // branch/tagの場合は--branchでclone
        Command::new("git")
            .args([
                "clone",
                "--branch",
                version,
                "https://github.com/Stephane-D/SGDK",
                target_dir.to_str().unwrap(),
            ])
            .status()
            .expect("git clone failed")
    };

    if !clone_status.success() {
        eprintln!("{}", rust_i18n::t!("git_clone_failed"));
        std::process::exit(1);
    }

    // commit IDの場合はcheckout
    if is_commit_id {
        let checkout_status = Command::new("git")
            .arg("checkout")
            .arg(version)
            .current_dir(&target_dir)
            .status()
            .expect("git checkout failed");
        if !checkout_status.success() {
            eprintln!("git checkout failed");
            std::process::exit(1);
        }
    }

    println!("{}", rust_i18n::t!("saving_config"));
    let config_dir = config_dir()
        .expect("Failed to get config directory")
        .join("sgdktool");
    fs::create_dir_all(&config_dir).expect("Failed to create config directory");
    let config_path = config_dir.join("config.toml");

    let mut doc = if config_path.exists() {
        let text = fs::read_to_string(&config_path).expect(&rust_i18n::t!("config_read_failed"));
        text.parse::<DocumentMut>()
            .expect(&rust_i18n::t!("toml_parse_failed"))
    } else {
        DocumentMut::new()
    };
    let abs_path = target_dir
        .canonicalize()
        .expect("Failed to get absolute path");
    doc["sgdk"]["path"] = value(abs_path.to_str().unwrap());
    doc["sgdk"]["version"] = value(version);

    fs::write(&config_path, doc.to_string()).expect("Failed to write config.toml");

    #[cfg(not(target_os = "windows"))]
    {
        run_generate_wine(&target_dir);
    }

    println!(
        "{}",
        rust_i18n::t!("sgdk_setup_complete", path = target_dir.display())
    );
}

#[cfg(not(target_os = "windows"))]
fn run_generate_wine(sgdk_path: &Path) {
    let sgdk_bin = sgdk_path.join("bin");
    let script_url =
        "https://raw.githubusercontent.com/Franticware/SGDK_wine/master/generate_wine.sh";
    let local_script = sgdk_bin.join("generate_wine.sh");

    println!("{}", rust_i18n::t!("wine_downloading"));
    let response = reqwest::blocking::get(script_url)
        .expect("Script download failed")
        .text()
        .expect("Text retrieval failed");
    fs::write(&local_script, response).expect("Failed to write generate_wine.sh");

    println!("{}", rust_i18n::t!("wine_generating"));
    let status = Command::new("sh")
        .arg("generate_wine.sh")
        .current_dir(sgdk_path.join("bin"))
        .status()
        .expect("Failed to execute generate_wine.sh");

    if !status.success() {
        eprintln!("{}", rust_i18n::t!("wine_script_failed"));
        std::process::exit(1);
    }

    println!("{}", rust_i18n::t!("wine_wrapper_complete"));
}
