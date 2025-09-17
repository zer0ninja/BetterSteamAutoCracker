// This module handles the initial setup for BetterSteamAutoCracker, downloading and caching dependencies
// from an S3 bucket into the app data directory (e.g., %APPDATA%/com.sovereign.better-steam-autocracker on Windows).
// Uses `reqwest` for HTTP requests and `std::fs` for filesystem operations, downloading files only if missing
// and extracting the Steamless ZIP for DRM removal. Runs asynchronously on startup.

// Managed dependencies:
// - steam_api(64).dll, steamclient(64).dll: 32/64-bit Goldberg Steam Emulator DLLs for Steam API emulation.
// - overlay_achievement_notification.wav: Audio for achievement notifications.
// - Roboto-Medium.ttf: Font for Steam-like UI rendering.
// - Steamless.v3.1.0.5.-.by.atom0s.zip: Steamless unpacker tool for removing Steam DRM.

use crate::config::{FOLDER, S3};
use dirs::data_dir;
use log::info;
use reqwest::Client;
use std::fs::{self, File};
use std::io::{self, Write};
use std::path::Path;
use tokio::task;
use zip::read::ZipArchive;

// Dependency URLs constructed at runtime using S3 base from config.rs
const GOLDBERG32_PATH: &str = "gameboun/x32/steam_api.dll";
const STEAMCLIENT32_PATH: &str = "gameboun/x32/steamclient.dll";
const GOLDBERG64_PATH: &str = "gameboun/x64/steam_api64.dll";
const STEAMCLIENT64_PATH: &str = "gameboun/x64/steamclient64.dll";
const OVERLAY_SOUND_PATH: &str = "gameboun/overlay_achievement_notification.wav";
const FONT_FILE_PATH: &str = "gameboun/Roboto-Medium.ttf";
const STEAMLESS_DOWNLOAD_PATH: &str = "gameboun/Steamless.v3.1.0.5.-.by.atom0s.zip";
const STEAMLESS_DIR_NAME: &str = "steamless";
const STEAMLESS_KEY_FILE: &str = "Steamless.CLI.exe";

// Struct for a downloadable file
struct DownloadableFile {
    url: String,
    target_path: String,
}

// Downloads a file if it doesn't exist
async fn download_file(client: Client, file: DownloadableFile) -> Result<(), String> {
    let target_path = Path::new(&file.target_path);
    if target_path.exists() {
        info!("File exists: {}", file.target_path);
        return Ok(());
    }

    // Create parent directories
    if let Some(parent) = target_path.parent() {
        fs::create_dir_all(parent).map_err(|e| format!("Failed to create directory: {}", e))?;
    }

    // Download file
    let bytes = client
        .get(&file.url)
        .send()
        .await
        .map_err(|e| format!("Failed to download {}: {}", file.url, e))?
        .bytes()
        .await
        .map_err(|e| format!("Failed to read bytes from {}: {}", file.url, e))?;

    // Write file
    let mut outfile = File::create(target_path)
        .map_err(|e| format!("Failed to create file {}: {}", file.target_path, e))?;
    outfile
        .write_all(&bytes)
        .map_err(|e| format!("Failed to write file {}: {}", file.target_path, e))?;

    info!("Downloaded: {}", file.target_path);
    Ok(())
}

// Extracts the Steamless ZIP if not already extracted
async fn extract_steamless_zip(cache_dir: &str) -> Result<(), String> {
    let zip_path = format!("{}/{}", cache_dir, "Steamless.v3.1.0.5.-.by.atom0s.zip");
    let extract_dir = format!("{}/{}", cache_dir, STEAMLESS_DIR_NAME);

    // Check if Steamless is already extracted
    if Path::new(&extract_dir).join(STEAMLESS_KEY_FILE).exists() {
        info!("Steamless already extracted: {}", extract_dir);
        return Ok(());
    }

    // Read ZIP file
    let zip_file =
        File::open(&zip_path).map_err(|e| format!("Failed to open ZIP {}: {}", zip_path, e))?;
    let mut archive =
        ZipArchive::new(zip_file).map_err(|e| format!("Failed to read ZIP {}: {}", zip_path, e))?;

    // Create extraction directory
    fs::create_dir_all(&extract_dir)
        .map_err(|e| format!("Failed to create directory {}: {}", extract_dir, e))?;

    // Extract all files
    for i in 0..archive.len() {
        let mut file = archive
            .by_index(i)
            .map_err(|e| format!("Failed to read ZIP entry {}: {}", i, e))?;
        let file_path = file
            .enclosed_name()
            .ok_or_else(|| format!("Invalid file path in ZIP entry {}", i))?;
        let target_path = Path::new(&extract_dir).join(file_path);

        if file.is_dir() {
            fs::create_dir_all(&target_path).map_err(|e| {
                format!(
                    "Failed to create directory {}: {}",
                    target_path.display(),
                    e
                )
            })?;
        } else {
            if let Some(parent) = target_path.parent() {
                fs::create_dir_all(parent).map_err(|e| {
                    format!(
                        "Failed to create parent directory {}: {}",
                        parent.display(),
                        e
                    )
                })?;
            }
            let mut outfile = File::create(&target_path)
                .map_err(|e| format!("Failed to create file {}: {}", target_path.display(), e))?;
            io::copy(&mut file, &mut outfile)
                .map_err(|e| format!("Failed to extract file {}: {}", target_path.display(), e))?;
        }
    }

    // Remove ZIP file
    fs::remove_file(&zip_path).map_err(|e| format!("Failed to remove ZIP {}: {}", zip_path, e))?;

    info!("Extracted Steamless to: {}", extract_dir);
    Ok(())
}

pub async fn setup(_app_handle: tauri::AppHandle) -> Result<(), String> {
    // Resolve cache directory
    let cache_dir = data_dir()
        .ok_or("Failed to get app data directory")?
        .join(FOLDER)
        .to_string_lossy()
        .into_owned();

    // Create cache directory
    fs::create_dir_all(&cache_dir)
        .map_err(|e| format!("Failed to create cache directory {}: {}", cache_dir, e))?;

    // Initialize reqwest client
    let client = Client::new();

    // List of files to download
    let files = vec![
        DownloadableFile {
            url: format!("{}{}", S3, GOLDBERG32_PATH),
            target_path: format!("{}/{}", cache_dir, "steam_api.dll"),
        },
        DownloadableFile {
            url: format!("{}{}", S3, STEAMCLIENT32_PATH),
            target_path: format!("{}/{}", cache_dir, "steamclient.dll"),
        },
        DownloadableFile {
            url: format!("{}{}", S3, GOLDBERG64_PATH),
            target_path: format!("{}/{}", cache_dir, "steam_api64.dll"),
        },
        DownloadableFile {
            url: format!("{}{}", S3, STEAMCLIENT64_PATH),
            target_path: format!("{}/{}", cache_dir, "steamclient64.dll"),
        },
        DownloadableFile {
            url: format!("{}{}", S3, OVERLAY_SOUND_PATH),
            target_path: format!("{}/{}", cache_dir, "overlay_achievement_notification.wav"),
        },
        DownloadableFile {
            url: format!("{}{}", S3, FONT_FILE_PATH),
            target_path: format!("{}/{}", cache_dir, "Roboto-Medium.ttf"),
        },
        DownloadableFile {
            url: format!("{}{}", S3, STEAMLESS_DOWNLOAD_PATH),
            target_path: format!("{}/{}", cache_dir, "Steamless.v3.1.0.5.-.by.atom0s.zip"),
        },
    ];

    // Download files concurrently by moving ownership
    let download_tasks: Vec<_> = files
        .into_iter()
        .map(|file| {
            let client = client.clone();
            task::spawn(async move { download_file(client, file).await })
        })
        .collect();

    for task in download_tasks {
        task.await
            .map_err(|e| format!("Download task failed: {}", e))?
            .map_err(|e| e)?;
    }

    // Extract Steamless ZIP
    extract_steamless_zip(&cache_dir).await?;

    info!("Setup completed successfully");
    Ok(())
}
