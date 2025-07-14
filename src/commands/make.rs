use crate::commands::new::escape_path;
use crate::path;
use clap::Parser;
use rust_i18n;
use std::fs;
use std::path::Path;
use std::process::Command;
use toml_edit::DocumentMut;

#[derive(Parser)]
pub struct Args {
    /// Additional options to pass to make
    #[arg(last = true)]
    extra: Vec<String>,
}
impl Args {
    pub fn new(extra: Vec<String>) -> Self {
        Self { extra }
    }
}

pub fn run(args: &Args) {
    let extra = &args.extra;
    let dir = Path::new(".");
    if !dir.exists() {
        eprintln!("{}", rust_i18n::t!("project_dir_not_found"));
        std::process::exit(1);
    }

    let config_path = path::config_dir().join("config.toml");
    let doc = fs::read_to_string(&config_path)
        .unwrap()
        .parse::<DocumentMut>()
        .unwrap();
    let (sgdk_path_str, _) = crate::commands::new::get_sgdk_config(&doc);
    let sgdk_path = Path::new(sgdk_path_str.unwrap_or_else(|| {
        eprintln!("SGDK path not found in config.toml.");
        std::process::exit(1);
    }));

    // パス文字列を取得
    let sgdk_path_str = sgdk_path.to_str().unwrap();
    let escaped_sgdk_path = escape_path(sgdk_path_str);
    println!("Using SGDK path: {}", escaped_sgdk_path);

    #[cfg(target_os = "windows")]
    let makefile = sgdk_path.join("makefile.gen");
    #[cfg(not(target_os = "windows"))]
    let makefile = sgdk_path.join("makefile_wine.gen");

    let mut cmd = Command::new("make");
    cmd.current_dir(&dir)
        .arg(format!("GDK={}", escaped_sgdk_path))
        .arg("-f")
        .arg(&makefile);

    for arg in extra {
        cmd.arg(arg);
    }

    let status = cmd.status().expect("Failed to execute make");
    std::process::exit(status.code().unwrap_or(1));
}
