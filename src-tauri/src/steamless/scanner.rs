use crate::config::FOLDER;
use crate::steamless::config::{STEAMLESS_DIR_NAME, STEAMLESS_KEY_FILE};
use crate::steamless::files;
use crate::steamless::utils;
use dirs::data_dir;
use log::info;
use serde_json::json;
use std::path::{Path, PathBuf};
use tauri::{AppHandle, Emitter};
use walkdir::WalkDir;

pub async fn run_steamless_on_directory(
    app_handle: &AppHandle,
    game_path: &Path,
) -> Result<(), String> {
    let cache_dir = data_dir()
        .ok_or("Failed to get app data directory")?
        .join(FOLDER)
        .to_string_lossy()
        .into_owned();

    let steamless_dir = PathBuf::from(&cache_dir).join(STEAMLESS_DIR_NAME);
    let steamless_cli_path = steamless_dir.join(STEAMLESS_KEY_FILE);

    if !steamless_cli_path.exists() {
        return Err(format!(
            "Steamless CLI not found at: {}. Did anti-virus eat it?",
            steamless_cli_path.display()
        ));
    }

    info!(
        "Starting Steamless processing on game directory: {}",
        game_path.display()
    );

    if !game_path.is_dir() {
        return Err(format!(
            "Provided game path is not a valid directory: {}",
            game_path.display()
        ));
    }

    // Collect all .exe files first to calculate total count for progress.
    let mut exes: Vec<PathBuf> = Vec::new();
    for entry in WalkDir::new(game_path)
        .follow_links(false)
        .into_iter()
        .filter_map(Result::ok)
    {
        let path = entry.path();
        if path.is_file() && is_exe_file(path) {
            exes.push(path.to_path_buf());
        }
    }

    let exe_count = exes.len() as f32;
    let mut success_count = 0;

    if exe_count == 0.0 {
        info!("No EXE files found in the directory.");
        app_handle
            .emit(
                "crack-progress",
                &json!({"progress": 50, "message": "No EXE files found for Steamless"}),
            )
            .map_err(|e| format!("Failed to emit progress: {}", e))?;
        return Ok(());
    }

    let step = 50.0 / exe_count;
    let mut current_progress = 0.0;

    for path in exes {
        app_handle.emit("crack-progress", &json!({"progress": current_progress as u32, "message": format!("Processing EXE: {}", path.display())})).map_err(|e| format!("Failed to emit progress: {}", e))?;

        info!("Processing EXE: {}", path.display());

        if let Err(e) = utils::run_steamless_on_exe(&steamless_cli_path, &path).await {
            info!("Steamless failed on {}: {}", path.display(), e);
            current_progress += step;
            continue;
        }

        if let Err(e) = files::handle_unpacked_exe(&path) {
            info!("Post-unpack handling failed for {}: {}", path.display(), e);
        } else {
            success_count += 1;
            info!("Successfully unpacked: {}", path.display());
        }

        current_progress += step;

        app_handle.emit("crack-progress", &json!({"progress": current_progress as u32, "message": format!("Processed EXE: {}", path.display())})).map_err(|e| format!("Failed to emit progress: {}", e))?;
    }

    info!(
        "Steamless scan complete. Processed {} EXE files. Unpacked {} files.",
        exe_count as u32, success_count
    );

    if current_progress < 50.0 {
        app_handle
            .emit(
                "crack-progress",
                &json!({"progress": 50, "message": "Steamless completed"}),
            )
            .map_err(|e| format!("Failed to emit progress: {}", e))?;
    }

    Ok(())
}

fn is_exe_file(path: &Path) -> bool {
    path.extension()
        .map(|ext| ext.to_ascii_lowercase() == "exe")
        .unwrap_or(false)
}
