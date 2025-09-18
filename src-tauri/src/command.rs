use serde::{Deserialize, Serialize};
use serde_json::json;
use tauri::{command, AppHandle, Emitter, State};
use std::fs;
use std::path::PathBuf;
use std::sync::Mutex;
use dirs::data_dir;

use crate::goldberg::apply::apply_goldberg;
use crate::steamless::apply::apply_steamless;
use crate::settings::{Settings, Theme};

#[derive(Debug, Serialize, Deserialize)]
struct SteamAppDetails {
    #[serde(flatten)]
    data: std::collections::HashMap<String, AppDetail>,
}

#[derive(Debug, Serialize, Deserialize)]
struct AppDetail {
    success: bool,
    data: Option<AppData>,
}

#[derive(Debug, Serialize, Deserialize)]
struct AppData {
    drm_notice: Option<String>,
}

#[cfg(target_os = "windows")]
use winapi::um::winuser::MessageBeep;

#[command]
fn play_system_beep() {
    #[cfg(target_os = "windows")]
    unsafe {
        MessageBeep(0xFFFFFFFF);
    }
}

#[command]
pub async fn cmd_apply_crack(
    app: AppHandle,
    game_location: String,
    app_id: String,
    language: Option<String>,
) -> Result<String, String> {
    app.emit(
        "crack-progress",
        &json!({"progress": 0, "message": "Starting Steamless"}),
    )
    .map_err(|e| format!("Failed to emit progress: {}", e))?;

    // Apply Steamless DRM removal
    let steamless_result = apply_steamless(app.clone(), game_location.clone())
        .await
        .map_err(|e| format!("Steamless failed: {}", e))?;

    app.emit(
        "crack-progress",
        &json!({"progress": 50, "message": "Starting Goldberg"}),
    )
    .map_err(|e| format!("Failed to emit progress: {}", e))?;

    // Apply Goldberg Steam Emulator
    let goldberg_result = apply_goldberg(app.clone(), game_location, app_id, language)
        .await
        .map_err(|e| format!("Goldberg failed: {}", e))?;

    app.emit(
        "crack-progress",
        &json!({"progress": 100, "message": "Done"}),
    )
    .map_err(|e| format!("Failed to emit progress: {}", e))?;

    // Plays a system beep on success, might remove it later
    play_system_beep();

    Ok(format!("{}\n{}", steamless_result, goldberg_result))
}

#[command]
pub async fn cmd_check_drm(app_id: String) -> Result<String, String> {
    let url = format!(
        "https://store.steampowered.com/api/appdetails?appids={}&l=english",
        app_id
    );

    let client = reqwest::Client::new();
    let response = client
        .get(&url)
        .send()
        .await
        .map_err(|e| format!("Failed to fetch Steam API: {}", e))?;

    let data: SteamAppDetails = response
        .json()
        .await
        .map_err(|e| format!("Failed to parse JSON: {}", e))?;

    let app_detail = data
        .data
        .get(&app_id)
        .ok_or_else(|| format!("No data for App ID {}", app_id))?;

    if !app_detail.success || app_detail.data.is_none() {
        return Ok("No DRM information available".to_string());
    }

    let drm_notice = app_detail
        .data
        .as_ref()
        .and_then(|data| data.drm_notice.as_ref());

    match drm_notice {
        Some(notice) => {
            let has_denuvo = notice.to_lowercase().contains("denuvo");
            Ok(if has_denuvo {
                format!("App ID {} uses Denuvo DRM", app_id)
            } else {
                format!("App ID {} has DRM notice: {}", app_id, notice)
            })
        }
        None => Ok(format!("App ID {} has no DRM notice", app_id)),
    }
}

#[cfg(target_os = "windows")]
#[command]
pub fn cmd_get_windows_theme() -> Result<String, String> {
    use crate::settings::get_system_theme;
    Ok(match get_system_theme() {
        Theme::Light => "light".to_string(),
        Theme::Dark => "dark".to_string(),
    })
}

#[command]
pub fn cmd_get_settings(state: State<Mutex<Settings>>) -> Result<Settings, String> {
    let settings = state
        .lock()
        .map_err(|e| format!("Failed to lock settings: {}", e))?;
    Ok(settings.clone())
}

#[command]
pub fn cmd_set_settings(state: State<Mutex<Settings>>, new_settings: Settings) -> Result<(), String> {
    let mut settings = state
        .lock()
        .map_err(|e| format!("Failed to lock settings: {}", e))?;

    *settings = new_settings;

    let settings_path = data_dir()
        .map(|dir| dir.join("sovereign.bsac.app").join("settings").join("theme.json"))
        .unwrap_or_else(|| PathBuf::from("settings.json"));

    if let Some(parent) = settings_path.parent() {
        fs::create_dir_all(parent)
            .map_err(|e| format!("Failed to create directory {}: {}", parent.display(), e))?;
    }

    fs::write(
        &settings_path,
        serde_json::to_string(&*settings).map_err(|e| format!("Failed to serialize settings: {}", e))?,
    )
    .map_err(|e| format!("Failed to save settings to {}: {}", settings_path.display(), e))?;
    Ok(())
}