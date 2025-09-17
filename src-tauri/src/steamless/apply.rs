use crate::steamless::scanner;
use log::info;
use std::path::PathBuf;
use tauri::AppHandle;

pub async fn apply_steamless(
    app_handle: AppHandle,
    game_location: String,
) -> Result<String, String> {
    let game_path = PathBuf::from(&game_location);
    if !game_path.is_dir() {
        return Err(format!("Invalid game dir: {}", game_location));
    }

    info!("Starting Steamless for: {}", game_location);

    scanner::run_steamless_on_directory(&app_handle, &game_path).await?;

    Ok(format!(
        "Steamless DRM removal completed for: {}",
        game_location
    ))
}
