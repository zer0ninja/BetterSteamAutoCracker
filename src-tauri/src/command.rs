
use serde_json::json;
use tauri::{command, AppHandle, Emitter};

use crate::goldberg::apply::apply_goldberg;
use crate::steamless::apply::apply_steamless;

#[cfg(target_os = "windows")]
use winapi::um::winuser::MessageBeep;

#[command]
fn play_system_beep() {
    #[cfg(target_os = "windows")]
    unsafe {
        MessageBeep(0xFFFFFFFF);
    }
    #[cfg(not(target_os = "windows"))]
    {
        // No-op on non-Windows platforms
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
