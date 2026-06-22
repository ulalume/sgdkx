use std::path::PathBuf;

/// Install/config location, unified across platforms: `~/.sgdkx/data`
/// (home-relative, like cargo/rustup — short and consistent on macOS, Linux, and Windows).
pub fn config_dir() -> PathBuf {
    let home = dirs::home_dir().expect("Failed to get home directory");
    home.join(".sgdkx").join("data")
}

// Everything sgdkx installs lives at a fixed spot under `config_dir()`, so component paths
// are derived rather than stored. config.toml keeps only the one non-derivable fact: which
// SGDK native-build version is installed.

/// The installed SGDK directory (`<config>/SGDK`).
pub fn sgdk_dir() -> PathBuf {
    config_dir().join("SGDK")
}

/// The bundled gcc toolchain dir, if present. `None` on Windows, where the toolchain lives
/// inside the SGDK bundle's `bin/` (no separate component).
pub fn toolchain_dir() -> Option<PathBuf> {
    let d = config_dir().join("m68k-elf-toolchain");
    d.join("bin").is_dir().then_some(d)
}

/// The bundled JRE dir, if present (else the build falls back to system `java`).
pub fn jre_dir() -> Option<PathBuf> {
    let d = config_dir().join("jre");
    d.join("bin").is_dir().then_some(d)
}

/// Whether `sgdkx install` has populated the environment (SGDK is present).
pub fn is_installed() -> bool {
    sgdk_dir().join("bin").is_dir()
}

/// The installed SGDK native-build version recorded at install time (config.toml's only
/// field), if any.
pub fn installed_version() -> Option<String> {
    let text = std::fs::read_to_string(config_dir().join("config.toml")).ok()?;
    let doc: toml_edit::DocumentMut = text.parse().ok()?;
    doc.get("sgdk")?
        .as_inline_table()?
        .get("version")?
        .as_str()
        .map(str::to_string)
}
