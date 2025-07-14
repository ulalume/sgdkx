use std::path::PathBuf;

#[cfg(target_os = "macos")]
pub fn config_dir() -> PathBuf {
    // macOS: ~/.sgdktool
    let home = dirs::home_dir().expect("Failed to get home directory");
    home.join(".sgdktool/data")
}

#[cfg(target_os = "windows")]
pub fn config_dir() -> PathBuf {
    // Windows: %APPDATA%\sgdktool
    let base = dirs::data_dir().expect("Failed to get data directory");
    base.join("sgdktool/data")
}

#[cfg(target_os = "linux")]
pub fn config_dir() -> PathBuf {
    // Linux: ~/.config/sgdktool
    let base = dirs::config_dir().expect("Failed to get config directory");
    base.join("sgdktool/data")
}

// その他のUnix (例: FreeBSDなど)
#[cfg(all(unix, not(any(target_os = "macos", target_os = "linux"))))]
pub fn config_dir() -> PathBuf {
    let home = dirs::home_dir().expect("Failed to get home directory");
    home.join(".sgdktool/data")
}
