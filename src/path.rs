use std::path::PathBuf;

/// Install/config location, unified across platforms: `~/.sgdkx/data`
/// (home-relative, like cargo/rustup — short and consistent on macOS, Linux, and Windows).
pub fn config_dir() -> PathBuf {
    let home = dirs::home_dir().expect("Failed to get home directory");
    home.join(".sgdkx").join("data")
}
