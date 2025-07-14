use crate::path;
use which::which;

/// SGDKドキュメントの表示
pub fn run() {
    // config_dirがResultを返すので、unwrap_or_elseでエラー時のデフォルト値を設定
    let sgdk_path = path::config_dir().join("SGDK");
    let out_html = sgdk_path.join("doc").join("html");
    let index_html = out_html.join("index.html");

    if out_html.exists() && out_html.is_dir() {
        println!(
            "{}",
            rust_i18n::t!("sgdk_doc_exists", path = out_html.display())
        );
        // index.htmlがあればブラウザで開く
        if index_html.exists() {
            #[cfg(target_os = "macos")]
            {
                let _ = std::process::Command::new("open").arg(&index_html).status();
            }
            #[cfg(target_os = "windows")]
            {
                let _ = std::process::Command::new("cmd")
                    .args(&["/C", "start", "", &index_html.to_string_lossy()])
                    .status();
            }
            #[cfg(all(unix, not(target_os = "macos")))]
            {
                let _ = std::process::Command::new("xdg-open")
                    .arg(&index_html)
                    .status();
            }
        } else {
            println!("index.html not found in doc directory.");
        }
        return;
    }

    // doxygenの有無を確認
    if which("doxygen").is_err() {
        println!("{}", rust_i18n::t!("sgdk_doc_doxygen_missing"));
    } else {
        println!("{}", rust_i18n::t!("sgdk_doc_not_generated"));
    }
}
