use crate::config::{FOLDER, S3};
use crate::error::SetupError;

use tauri::Emitter;

use dirs::data_dir;
use futures_util::stream::StreamExt;
use log::info;
use reqwest::Client;
use std::fs::{self, File};
use std::io;
use std::path::Path;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Semaphore;
use tokio::task::{self, spawn_blocking};
use tokio::io::AsyncWriteExt;
use zip::read::ZipArchive;

const GOLDBERG32_PATH: &str = "gameboun/x32/steam_api.dll";
const STEAMCLIENT32_PATH: &str = "gameboun/x32/steamclient.dll";
const GOLDBERG64_PATH: &str = "gameboun/x64/steam_api64.dll";
const STEAMCLIENT64_PATH: &str = "gameboun/x64/steamclient64.dll";
const OVERLAY_SOUND_PATH: &str = "gameboun/overlay_achievement_notification.wav";
const FONT_FILE_PATH: &str = "gameboun/Roboto-Medium.ttf";
const STEAMLESS_DOWNLOAD_PATH: &str = "gameboun/Steamless.v3.1.0.5.-.by.atom0s.zip";
const STEAMLESS_DIR_NAME: &str = "steamless";
const STEAMLESS_KEY_FILE: &str = "Steamless.CLI.exe";

struct DownloadableFile {
    url: String,
    target_path: String,
}

async fn download_file(
    client: Client,
    file: DownloadableFile,
    semaphore: &Semaphore,
    app_handle: &tauri::AppHandle,
) -> Result<(), SetupError> {
    let target_path = Path::new(&file.target_path);
    if target_path.exists() {
        info!("File exists: {}", file.target_path);
        app_handle.emit("setup-progress", format!("Skipped: {}", file.target_path))?;
        return Ok(());
    }

    if let Some(parent) = target_path.parent() {
        fs::create_dir_all(parent)?;
    }

    let _permit = semaphore
        .acquire()
        .await
        .map_err(|e| SetupError::Other(format!("Semaphore error: {}", e)))?;
    app_handle.emit("setup-progress", format!("Downloading: {}", file.target_path))?;

    let response = client
        .get(&file.url)
        .timeout(Duration::from_secs(30))
        .send()
        .await?;

    let mut outfile = tokio::fs::File::create(&file.target_path).await?;
    let mut stream = response.bytes_stream();

    while let Some(chunk) = stream.next().await {
        let chunk = chunk?;
        outfile.write_all(&chunk).await?;
    }

    info!("Downloaded: {}", file.target_path);
    app_handle.emit("setup-progress", format!("Completed: {}", file.target_path))?;
    Ok(())
}

async fn extract_steamless_zip(cache_dir: &str, app_handle: &tauri::AppHandle) -> Result<(), SetupError> {
    let zip_path = format!("{}/{}", cache_dir, "Steamless.v3.1.0.5.-.by.atom0s.zip");
    let extract_dir = format!("{}/{}", cache_dir, STEAMLESS_DIR_NAME);

    if Path::new(&extract_dir).join(STEAMLESS_KEY_FILE).exists() {
        info!("Steamless already extracted: {}", extract_dir);
        app_handle.emit("setup-progress", "Steamless already extracted")?;
        return Ok(());
    }

    fs::create_dir_all(&extract_dir)?;

    let zip_path_clone = zip_path.clone();
    let extract_dir_clone = extract_dir.clone();

    app_handle.emit("setup-progress", "Extracting Steamless ZIP")?;
    spawn_blocking(move || {
        let zip_file = File::open(&zip_path_clone)?;
        let mut archive = ZipArchive::new(zip_file)?;

        for i in 0..archive.len() {
            let mut file = archive.by_index(i)?;
            let file_path = file
                .enclosed_name()
                .ok_or_else(|| SetupError::Other(format!("Invalid file path in ZIP entry {}", i)))?;
            let target_path = Path::new(&extract_dir_clone).join(file_path);

            if file.is_dir() {
                fs::create_dir_all(&target_path)?;
            } else {
                if let Some(parent) = target_path.parent() {
                    fs::create_dir_all(parent)?;
                }
                let mut outfile = File::create(&target_path)?;
                io::copy(&mut file, &mut outfile)?;
            }
        }
        Ok::<(), SetupError>(())
    })
    .await??;

    fs::remove_file(&zip_path)?;
    info!("Extracted Steamless to: {}", extract_dir);
    app_handle.emit("setup-progress", "Steamless extraction completed")?;
    Ok(())
}

pub async fn setup(app_handle: tauri::AppHandle) -> Result<(), SetupError> {
    let cache_dir = data_dir()
        .ok_or(SetupError::Other("Failed to get app data directory".to_string()))?
        .join(FOLDER)
        .to_string_lossy()
        .into_owned();

    fs::create_dir_all(&cache_dir)?;
    app_handle.emit("setup-progress", "Created cache directory")?;

    let client = Client::new();
    let semaphore = Arc::new(Semaphore::new(3));

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

    let download_tasks: Vec<_> = files
        .into_iter()
        .map(|file| {
            let client = client.clone();
            let semaphore = semaphore.clone();
            let app_handle = app_handle.clone();
            task::spawn(async move { download_file(client, file, &semaphore, &app_handle).await })
        })
        .collect();

    for task in download_tasks {
        task.await??;
    }

    extract_steamless_zip(&cache_dir, &app_handle).await?;
    info!("Setup completed");
    app_handle.emit("setup-progress", "Setup completed")?;
    Ok(())
}