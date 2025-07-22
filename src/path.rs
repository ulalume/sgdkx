use std::path::PathBuf;

#[cfg(target_os = "macos")]
pub fn config_dir() -> PathBuf {
    // macOS: ~/.sgdkx
    let base = dirs::home_dir().expect("Failed to get home directory");
    base.join(".sgdkx").join("data")
}

#[cfg(target_os = "windows")]
pub fn config_dir() -> PathBuf {
    let base = dirs::document_dir().expect("Failed to get data directory");
    base.join(".sgdkx").join("data")
}

#[cfg(target_os = "linux")]
pub fn config_dir() -> PathBuf {
    // Linux: ~/.config/sgdkx
    let base = dirs::config_dir().expect("Failed to get config directory");
    base.join("sgdkx").join("data")
}

// その他のUnix (例: FreeBSDなど)
#[cfg(all(unix, not(any(target_os = "macos", target_os = "linux"))))]
pub fn config_dir() -> PathBuf {
    let home = dirs::home_dir().expect("Failed to get home directory");
    home.join(".sgdkx").join("data")
}
