use clap::Parser;
use reqwest::blocking::Client;
use serde::{Deserialize, Serialize};
use std::fs;
use std::io::{self, Cursor};
use std::path::PathBuf;
use toml_edit::{DocumentMut, value};
use zip::ZipArchive;

// 多言語化
use rust_i18n;

// Constants for configuration paths
pub const SGDKTOOL_CONFIG_DIR_NAME: &str = "sgdktool";
pub const WEB_EXPORT_DIR_NAME: &str = "web-export";
const CONFIG_FILE_NAME: &str = "config.toml";
const WEB_TEMPLATE_GITHUB_API_URL: &str = "https://api.github.com/repos/ulalume/sgdktool/releases";
// Alternative URL for direct download if GitHub API fails
const WEB_TEMPLATE_DIRECT_URL: &str =
    "https://github.com/ulalume/sgdktool/releases/download/v0.0.1/web-template-v0.0.1.zip";

// GitHub API response structures
#[derive(Serialize, Deserialize, Debug)]
struct GithubRelease {
    tag_name: String,
    assets: Vec<GithubAsset>,
}

#[derive(Serialize, Deserialize, Debug)]
struct GithubAsset {
    name: String,
    browser_download_url: String,
}

#[derive(Parser)]
pub struct Args {}

pub fn run(_args: &Args) {
    let client = Client::new();
    let config_dir = dirs::config_dir()
        .expect("Failed to get config directory")
        .join(SGDKTOOL_CONFIG_DIR_NAME);
    let web_export_template_dir = config_dir.join(WEB_EXPORT_DIR_NAME);
    let config_path = config_dir.join(CONFIG_FILE_NAME);

    // Ensure config directory exists
    fs::create_dir_all(&config_dir).expect("Failed to create config directory");

    println!("{}", rust_i18n::t!("fetching_releases"));

    // Add User-Agent header to avoid GitHub API restrictions
    let response = match client
        .get(WEB_TEMPLATE_GITHUB_API_URL)
        .header("User-Agent", "sgdktool/0.1.1")
        .header("Accept", "application/vnd.github.v3+json")
        .send()
    {
        Ok(resp) => resp,
        Err(e) => {
            eprintln!("❌ Failed to fetch GitHub releases: {}", e);
            eprintln!("Check your internet connection and try again.");
            std::process::exit(1);
        }
    };

    // Try to extract response text for better error handling
    if !response.status().is_success() {
        let status = response.status();
        let body = response
            .text()
            .unwrap_or_else(|_| String::from("[Could not read response body]"));
        eprintln!("❌ GitHub API returned error ({}): {}", status, body);
        std::process::exit(1);
    }

    // Try to get response text for debugging
    let response_text = match response.text() {
        Ok(text) => text,
        Err(e) => {
            eprintln!("❌ Failed to read GitHub API response: {}", e);
            eprintln!("Falling back to direct template download...");
            // Skip JSON parsing and use direct template URL
            download_and_extract_template(
                client,
                WEB_TEMPLATE_DIRECT_URL,
                "v0.0.1",
                web_export_template_dir,
                config_path,
            );
            return;
        }
    };

    // Debug print if needed
    // eprintln!("Response: {}", response_text);

    // Parse JSON response
    let releases: Vec<GithubRelease> = match serde_json::from_str(&response_text) {
        Ok(rel) => rel,
        Err(e) => {
            eprintln!("❌ Failed to parse GitHub releases JSON: {}", e);
            eprintln!("This might be due to GitHub API rate limiting or format changes.");
            eprintln!(
                "Response starts with: {}",
                &response_text.chars().take(100).collect::<String>()
            );
            eprintln!("Falling back to direct template download...");
            // Skip JSON parsing and use direct template URL
            download_and_extract_template(
                client,
                WEB_TEMPLATE_DIRECT_URL,
                "v0.0.1",
                web_export_template_dir,
                config_path,
            );
            return;
        }
    };

    if releases.is_empty() {
        eprintln!("❌ No releases found in GitHub repository");
        eprintln!("Falling back to direct template download...");
        // Skip GitHub API and use direct template URL
        download_and_extract_template(
            client,
            WEB_TEMPLATE_DIRECT_URL,
            "v0.0.1",
            web_export_template_dir,
            config_path,
        );
        return;
    }

    let latest_release = &releases[0]; // GitHub API returns releases in descending order by date
    let latest_tag_name = &latest_release.tag_name;
    let zipball_url = &latest_release.assets.first().unwrap().browser_download_url;

    println!(
        "{}: {}",
        rust_i18n::t!("latest_template_version"),
        latest_tag_name
    );
    // Download and extract the template from GitHub releases
    download_and_extract_template(
        client,
        zipball_url,
        latest_tag_name,
        web_export_template_dir,
        config_path,
    );
}

// Helper function to download and extract template
fn download_and_extract_template(
    client: Client,
    zipball_url: &str,
    _tag_name: &str, // Prefix with underscore to indicate intentional non-use
    web_export_template_dir: PathBuf,
    config_path: PathBuf,
) {
    println!("{}", rust_i18n::t!("downloading_template"));

    let mut zip_response = match client
        .get(zipball_url)
        .header("User-Agent", "sgdktool/0.1.1")
        .send()
    {
        Ok(resp) => {
            if !resp.status().is_success() {
                eprintln!(
                    "❌ Failed to download zipball: HTTP status {}",
                    resp.status()
                );
                std::process::exit(1);
            }
            resp
        }
        Err(e) => {
            eprintln!("❌ Failed to download zipball: {}", e);
            std::process::exit(1);
        }
    };

    let mut zip_bytes = Vec::new();
    match zip_response.copy_to(&mut zip_bytes) {
        Ok(_) => {}
        Err(e) => {
            eprintln!("❌ Failed to read zipball data: {}", e);
            std::process::exit(1);
        }
    }

    let cursor = Cursor::new(zip_bytes);
    let mut archive = match ZipArchive::new(cursor) {
        Ok(a) => a,
        Err(e) => {
            eprintln!("❌ Failed to open zip archive: {}", e);
            eprintln!("The downloaded file might be corrupted or not a valid zip file.");
            std::process::exit(1);
        }
    };

    // Clear existing web-export directory before extraction
    if web_export_template_dir.exists() {
        if let Err(e) = fs::remove_dir_all(&web_export_template_dir) {
            eprintln!("❌ Failed to remove existing web-export directory: {}", e);
            std::process::exit(1);
        }
    }

    if let Err(e) = fs::create_dir_all(&web_export_template_dir) {
        eprintln!("❌ Failed to create web-export directory: {}", e);
        std::process::exit(1);
    }

    println!("{}", rust_i18n::t!("extracting_template"));
    for i in 0..archive.len() {
        let mut file = match archive.by_index(i) {
            Ok(f) => f,
            Err(e) => {
                eprintln!("⚠️ Warning: Failed to access zip entry {}: {}", i, e);
                continue;
            }
        };

        // Skip macOS metadata files and other hidden files
        let file_name = file.name();
        if file_name.contains("__MACOSX/")
            || file_name.contains(".DS_Store")
            || file_name.starts_with("._")
            || file_name.contains("/._")
        {
            continue;
        }

        let outpath = match file.enclosed_name() {
            Some(path) => {
                let components: Vec<_> = path.components().collect();
                if components.len() > 1 {
                    // Skip the top-level directory (e.g., sgdktool-web-template-0.0.1/...)
                    let relative_path: PathBuf = components[1..].iter().collect();
                    // Additional check for macOS metadata on the relative path
                    let rel_path_str = relative_path.to_string_lossy();
                    if rel_path_str.starts_with("._") || rel_path_str.contains(".DS_Store") {
                        continue;
                    }
                    web_export_template_dir.join(relative_path)
                } else {
                    continue; // Skip the top-level directory itself
                }
            }
            None => continue,
        };

        if (*file.name()).ends_with('/') {
            if let Err(e) = fs::create_dir_all(&outpath) {
                eprintln!(
                    "⚠️ Warning: Failed to create directory {}: {}",
                    outpath.display(),
                    e
                );
                continue;
            }
        } else {
            if let Some(p) = outpath.parent() {
                if !p.exists() {
                    if let Err(e) = fs::create_dir_all(p) {
                        eprintln!(
                            "⚠️ Warning: Failed to create parent directory {}: {}",
                            p.display(),
                            e
                        );
                        continue;
                    }
                }
            }

            let mut outfile = match fs::File::create(&outpath) {
                Ok(f) => f,
                Err(e) => {
                    eprintln!(
                        "⚠️ Warning: Failed to create file {}: {}",
                        outpath.display(),
                        e
                    );
                    continue;
                }
            };

            if let Err(e) = io::copy(&mut file, &mut outfile) {
                eprintln!(
                    "⚠️ Warning: Failed to write file {}: {}",
                    outpath.display(),
                    e
                );
            }
        }
    }

    // Update config.toml
    let mut doc = if config_path.exists() {
        match fs::read_to_string(&config_path) {
            Ok(text) => match text.parse::<DocumentMut>() {
                Ok(doc) => doc,
                Err(e) => {
                    eprintln!("{}: {}", rust_i18n::t!("toml_parse_failed"), e);
                    std::process::exit(1);
                }
            },
            Err(e) => {
                eprintln!("{}: {}", rust_i18n::t!("config_read_failed"), e);
                std::process::exit(1);
            }
        }
    } else {
        DocumentMut::new()
    };

    // Make sure web_export section exists
    if doc.get("web_export").is_none() {
        doc["web_export"] = toml_edit::table();
    }

    // Get absolute path and convert to string
    let abs_path = match web_export_template_dir.canonicalize() {
        Ok(p) => p,
        Err(e) => {
            eprintln!(
                "❌ Failed to get absolute path for template directory: {}",
                e
            );
            web_export_template_dir.clone() // Clone to avoid ownership issues
        }
    };

    let path_str = match abs_path.to_str() {
        Some(s) => s,
        None => {
            eprintln!("⚠️ Warning: Path contains invalid Unicode, using relative path instead");
            match web_export_template_dir.to_str() {
                Some(s) => s,
                None => {
                    eprintln!("❌ Critical: Path is not valid Unicode");
                    std::process::exit(1);
                }
            }
        }
    };

    doc["web_export"]["template_path"] = value(path_str.replace(r"\\?\", ""));

    if let Err(e) = fs::write(&config_path, doc.to_string()) {
        eprintln!("❌ Failed to write config file: {}", e);
        std::process::exit(1);
    }

    println!(
        "{}",
        rust_i18n::t!(
            "web_template_setup_complete",
            path = web_export_template_dir.display()
        )
    );
}

// Function removed, replaced with direct JSON parsing
