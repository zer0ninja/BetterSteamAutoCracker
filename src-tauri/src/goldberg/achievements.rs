use crate::config::STEAM_API_KEY;
use log::info;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::fs::File;
use std::io::Write;
use std::path::Path;
use tauri::{AppHandle, Emitter};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DlcEntry {
    pub id: String,
    pub name: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Achievement {
    pub name: String,
    pub display_name: String,
    pub description: String,
    pub hidden: bool,
    pub icon: String,
    pub icon_gray: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct Stat {
    name: String,
    #[serde(rename = "type")]
    stat_type: String,
    default: i32,
    global: i32,
}

#[derive(Debug, Deserialize)]
struct SteamGameSchema {
    game: Option<SteamGame>,
}

#[derive(Debug, Deserialize)]
struct SteamGame {
    #[serde(rename = "availableGameStats")]
    available_game_stats: Option<AvailableGameStats>,
}

#[derive(Debug, Deserialize)]
struct AvailableGameStats {
    achievements: Vec<SteamAchievement>,
    stats: Vec<SteamStat>,
}

#[derive(Debug, Deserialize)]
struct SteamAchievement {
    name: String,
    #[serde(rename = "displayName")]
    display_name: String,
    hidden: i32,
    description: Option<String>,
    icon: String,
    icongray: String,
    #[serde(rename = "defaultvalue")]
    default_value: i32,
}

#[derive(Debug, Deserialize)]
struct SteamStat {
    name: String,
    #[serde(rename = "defaultvalue")]
    default_value: i32,
    #[serde(rename = "displayName")]
    display_name: String,
}

pub async fn fetch_achievements(
    app_handle: &AppHandle,
    app_id: &str,
    language: Option<String>,
    steam_settings_dir: &Path,
    dll_step: f32,
    substeps_per_dll: f32,
) -> Result<(), String> {
    let client = Client::builder()
        .build()
        .map_err(|e| format!("Failed to create HTTP client: {}", e))?;

    let lang = language.unwrap_or_else(|| "english".to_string());
    let url = format!(
        "https://api.steampowered.com/ISteamUserStats/GetSchemaForGame/v2/?key={}&appid={}&l={}",
        STEAM_API_KEY, app_id, lang
    );

    info!(
        "Fetching achievements for AppID: {} at URL: {}",
        app_id, url
    );
    app_handle.emit("crack-progress", &json!({"progress": 50, "message": format!("Fetching achievements for AppID: {}", app_id)})).map_err(|e| format!("Failed to emit progress: {}", e))?;

    let response = client
        .get(&url)
        .send()
        .await
        .map_err(|e| format!("Failed to fetch achievements: {}", e))?;

    if !response.status().is_success() {
        return Err(format!(
            "Failed to fetch achievements: HTTP status {}",
            response.status()
        ));
    }

    let body = response
        .text()
        .await
        .map_err(|e| format!("Failed to read response body: {}", e))?;

    let schema: SteamGameSchema = serde_json::from_str(&body).map_err(|e| {
        format!(
            "Failed to parse Steam API response: {}",
            e
        )
    })?;

    let (achievements, stats) = if let Some(game) = schema.game {
        if let Some(stats_data) = game.available_game_stats {
            let achievements: Vec<Achievement> = stats_data
                .achievements
                .into_iter()
                .map(|ach| {
                    info!(
                        "Processing achievement '{}': icon={}, default_value={}, description={:?}",
                        ach.name, ach.icon, ach.default_value, ach.description
                    );
                    Achievement {
                        name: ach.name,
                        display_name: ach.display_name,
                        description: ach
                            .description
                            .unwrap_or_else(|| "No description available".to_string()),
                        hidden: ach.hidden != 0,
                        icon: ach.icon.clone(),
                        icon_gray: ach.icongray.clone(),
                    }
                })
                .collect();

            let stats: Vec<Stat> = stats_data
                .stats
                .into_iter()
                .map(|stat| {
                    info!(
                        "Processing stat '{}': default_value={}, display_name={}",
                        stat.name, stat.default_value, stat.display_name
                    );
                    Stat {
                        name: stat.name,
                        stat_type: "int".to_string(),
                        default: stat.default_value,
                        global: 0,
                    }
                })
                .collect();

            (achievements, stats)
        } else {
            info!("No achievements or stats found for AppID: {}", app_id);
            (Vec::new(), Vec::new())
        }
    } else {
        info!("No game data in Steam API response for AppID: {}", app_id);
        (Vec::new(), Vec::new())
    };

    let total_items =
        achievements.len() as f32 + stats.len() as f32 + (achievements.len() as f32 * 2.0); // Each achievement has icon and icon_gray
    let progress_per_item = (3.0 * dll_step / substeps_per_dll) / total_items;
    let mut current_subprogress = 0.0;

    // Save achievements.json
    let achievements_path = steam_settings_dir.join("achievements.json");
    for ach in achievements.iter() {
        current_subprogress += progress_per_item;
        app_handle.emit("crack-progress", &json!({"progress": (50.0 + current_subprogress) as u32, "message": format!("Processing achievement: {}", ach.display_name)})).map_err(|e| format!("Failed to emit progress: {}", e))?;
    }
    let achievements_json = serde_json::to_string_pretty(&achievements)
        .map_err(|e| format!("Failed to serialize achievements.json: {}", e))?;
    let mut file = File::create(&achievements_path).map_err(|e| {
        format!(
            "Failed to create achievements.json at {}: {}",
            achievements_path.display(),
            e
        )
    })?;
    file.write_all(achievements_json.as_bytes())
        .map_err(|e| format!("Failed to write achievements.json: {}", e))?;
    file.sync_all()
        .map_err(|e| format!("Failed to sync achievements.json: {}", e))?;
    info!("Saved achievements to: {}", achievements_path.display());
    current_subprogress += progress_per_item;
    app_handle.emit("crack-progress", &json!({"progress": (50.0 + current_subprogress) as u32, "message": "Saved achievements.json"})).map_err(|e| format!("Failed to emit progress: {}", e))?;

    // Save stats.json
    let stats_path = steam_settings_dir.join("stats.json");
    for stat in stats.iter() {
        current_subprogress += progress_per_item;
        app_handle.emit("crack-progress", &json!({"progress": (50.0 + current_subprogress) as u32, "message": format!("Processing stat: {}", stat.name)})).map_err(|e| format!("Failed to emit progress: {}", e))?;
    }
    let stats_json = serde_json::to_string_pretty(&stats)
        .map_err(|e| format!("Failed to serialize stats.json: {}", e))?;
    let mut file = File::create(&stats_path).map_err(|e| {
        format!(
            "Failed to create stats.json at {}: {}",
            stats_path.display(),
            e
        )
    })?;
    file.write_all(stats_json.as_bytes())
        .map_err(|e| format!("Failed to write stats.json: {}", e))?;
    file.sync_all()
        .map_err(|e| format!("Failed to sync stats.json: {}", e))?;
    info!("Saved stats to: {}", stats_path.display());
    current_subprogress += progress_per_item;
    app_handle.emit("crack-progress", &json!({"progress": (50.0 + current_subprogress) as u32, "message": "Saved stats.json"})).map_err(|e| format!("Failed to emit progress: {}", e))?;

    // Download achievement icons
    let images_dir = steam_settings_dir.join("images");
    std::fs::create_dir_all(&images_dir).map_err(|e| {
        format!(
            "Failed to create images directory {}: {}",
            images_dir.display(),
            e
        )
    })?;
    current_subprogress += progress_per_item;
    app_handle.emit("crack-progress", &json!({"progress": (50.0 + current_subprogress) as u32, "message": "Created images directory"})).map_err(|e| format!("Failed to emit progress: {}", e))?;

    for ach in &achievements {
        for (original_url, field_name) in &[(&ach.icon, "icon"), (&ach.icon_gray, "icon_gray")] {
            let icon_file = original_url.split('/').last().unwrap_or("unknown.jpg");
            let icon_path = images_dir.join(icon_file);
            if !icon_path.exists() {
                info!(
                    "Downloading {}: {} for '{}'",
                    field_name, original_url, ach.name
                );
                app_handle.emit("crack-progress", &json!({"progress": (50.0 + current_subprogress) as u32, "message": format!("Downloading {} for achievement: {}", field_name, ach.display_name)})).map_err(|e| format!("Failed to emit progress: {}", e))?;
                let response = client.get(*original_url).send().await.map_err(|e| {
                    format!(
                        "Failed to download {} {} for '{}': {}",
                        field_name, original_url, ach.name, e
                    )
                })?;

                if !response.status().is_success() {
                    info!(
                        "Failed to download {} {} for '{}': HTTP status {}",
                        field_name,
                        original_url,
                        ach.name,
                        response.status()
                    );
                    continue;
                }

                let bytes = response.bytes().await.map_err(|e| {
                    format!(
                        "Failed to read {} {} for '{}': {}",
                        field_name, original_url, ach.name, e
                    )
                })?;
                let mut file = File::create(&icon_path).map_err(|e| {
                    format!(
                        "Failed to create {} file {} for '{}': {}",
                        field_name,
                        icon_path.display(),
                        ach.name,
                        e
                    )
                })?;
                file.write_all(&bytes).map_err(|e| {
                    format!(
                        "Failed to write {} {} for '{}': {}",
                        field_name,
                        icon_path.display(),
                        ach.name,
                        e
                    )
                })?;
                file.sync_all().map_err(|e| {
                    format!(
                        "Failed to sync {} file {} for '{}': {}",
                        field_name,
                        icon_path.display(),
                        ach.name,
                        e
                    )
                })?;
                info!(
                    "Downloaded {} to {} for '{}'",
                    field_name,
                    icon_path.display(),
                    ach.name
                );
                current_subprogress += progress_per_item;
                app_handle.emit("crack-progress", &json!({"progress": (50.0 + current_subprogress) as u32, "message": format!("Downloaded {} for achievement: {}", field_name, ach.display_name)})).map_err(|e| format!("Failed to emit progress: {}", e))?;
            }
        }
    }

    Ok(())
}
