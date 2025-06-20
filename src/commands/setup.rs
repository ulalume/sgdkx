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
        // 上書き確認プロンプト
        println!("{}", rust_i18n::t!("sgdk_exists_overwrite_prompt"));
        use std::io::{self, Write};
        print!("{}", rust_i18n::t!("prompt"));
        io::stdout().flush().unwrap();

        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();
        let input = input.trim().to_lowercase();

        if input != "y" {
            println!("{}", rust_i18n::t!("sgdk_overwrite_cancelled"));
            std::process::exit(0);
        }

        // mac/linuxの場合はワーキングツリーを徹底的にクリーンにする
        #[cfg(not(target_os = "windows"))]
        {
            reset_sgdk_worktree(&target_dir);
        }
        // 既存リポジトリでgit fetch/checkout
        if target_dir.join(".git").exists() {
            let fetch_status = Command::new("git")
                .arg("fetch")
                .current_dir(&target_dir)
                .status()
                .expect(&rust_i18n::t!("sgdk_git_fetch_failed"));
            if !fetch_status.success() {
                eprintln!("{}", rust_i18n::t!("sgdk_git_fetch_failed"));
                std::process::exit(1);
            }
            let checkout_status = Command::new("git")
                .arg("checkout")
                .arg(version)
                .current_dir(&target_dir)
                .status()
                .expect(&rust_i18n::t!("sgdk_git_checkout_failed"));
            if !checkout_status.success() {
                eprintln!("{}", rust_i18n::t!("sgdk_git_checkout_failed"));
                std::process::exit(1);
            }
        } else {
            eprintln!("{}", rust_i18n::t!("sgdk_git_missing"));
            std::process::exit(1);
        }

        // config.toml更新
        println!("{}", rust_i18n::t!("sgdk_config_updating"));
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
        println!("{}", rust_i18n::t!("sgdk_config_updated"));

        #[cfg(not(target_os = "windows"))]
        {
            run_generate_wine(&target_dir);
        }

        generate_sgdk_doc(&target_dir);
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

    generate_sgdk_doc(&target_dir);

    println!(
        "{}",
        rust_i18n::t!("sgdk_setup_complete", path = target_dir.display())
    );
}

#[cfg(not(target_os = "windows"))]
fn reset_sgdk_worktree(target_dir: &Path) {
    use std::fs;
    // 1. git reset --hard
    let reset_status = std::process::Command::new("git")
        .args(&["reset", "--hard"])
        .current_dir(target_dir)
        .status();

    match reset_status {
        Ok(s) if s.success() => {
            println!("git reset --hard executed successfully.");
        }
        Ok(s) => {
            eprintln!("git reset --hard exited with code {:?}", s.code());
        }
        Err(e) => {
            eprintln!("failed to execute git reset --hard: {}", e);
        }
    }

    // 2. git clean -dfx .（2回実行）
    for i in 1..=2 {
        let clean_status = std::process::Command::new("git")
            .args(&["clean", "-dfx", "."])
            .current_dir(target_dir)
            .status();

        match clean_status {
            Ok(s) if s.success() => {
                println!("git clean -dfx . executed successfully (pass {}).", i);
            }
            Ok(s) => {
                eprintln!("git clean exited with code {:?} (pass {}).", s.code(), i);
            }
            Err(e) => {
                eprintln!("failed to execute git clean (pass {}): {}", i, e);
            }
        }
    }

    // 3. tools/sjasm ディレクトリを明示的に削除
    let sjasm_dir = target_dir.join("tools").join("sjasm");
    if sjasm_dir.exists() {
        println!("Removing problematic directory: {}", sjasm_dir.display());
        let _ = fs::remove_dir_all(&sjasm_dir);
    }
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

/// SGDKドキュメント生成（doxygenがある場合のみ）
fn generate_sgdk_doc(target_dir: &Path) {
    if which("doxygen").is_ok() {
        use regex::Regex;
        let doc_dir = target_dir.join("doc");
        let html_index = doc_dir.join("html").join("index.html");
        let doxyconfig = doc_dir.join("doxyconfig");
        let temp_path = doc_dir.join("temp_doxyconfig");

        // すでにdoc/html/index.htmlが存在する場合は何もしない
        if html_index.exists() {
            return;
        }

        // doc/doxyconfigがある場合のみ生成
        if doxyconfig.exists() {
            // 1. doxyconfig をコピー
            if let Err(e) = fs::copy(&doxyconfig, &temp_path) {
                eprintln!("Failed to copy doxygen config: {}", e);
                return;
            }

            // 2. ファイル内容を読み込み
            if let Ok(content) = fs::read_to_string(&temp_path) {
                // 3. OUTPUT_DIRECTORY の行を正規表現で置き換え
                let re = Regex::new(r"(?m)^OUTPUT_DIRECTORY\s*=.*$").unwrap();
                let new_content = re.replace_all(&content, "OUTPUT_DIRECTORY = ./SGDK/doc");
                // 4. 修正後の内容を書き戻し
                if let Err(e) = fs::write(&temp_path, new_content.as_ref()) {
                    eprintln!("Failed to write temp_doxyconfig: {}", e);
                } else {
                    // 5. doxygen を実行
                    let parent_dir = target_dir.parent().unwrap_or(target_dir);
                    let status = Command::new("doxygen")
                        .arg(&temp_path)
                        .current_dir(parent_dir)
                        .status();
                    match status {
                        Ok(s) if s.success() => {
                            println!("{}", rust_i18n::t!("sgdk_doc_generated"));
                        }
                        Ok(s) => {
                            eprintln!("doxygen exited with code {:?}", s.code());
                        }
                        Err(e) => {
                            eprintln!("failed to execute doxygen: {}", e);
                        }
                    }
                }
            }
        }
    }
}

// /// SGDKディレクトリをOSごとに開く
// fn open_sgdk_dir(sgdk_path: &Path) {
//     #[cfg(target_os = "macos")]
//     {
//         let _ = std::process::Command::new("open").arg(sgdk_path).status();
//     }
//     #[cfg(target_os = "windows")]
//     {
//         let _ = std::process::Command::new("explorer")
//             .arg(sgdk_path)
//             .status();
//     }
//     #[cfg(all(unix, not(target_os = "macos")))]
//     {
//         let _ = std::process::Command::new("xdg-open")
//             .arg(sgdk_path)
//             .status();
//     }
// }
