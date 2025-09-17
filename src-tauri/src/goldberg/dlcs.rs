// I'm not exactly sure if this'll work all the time, as some games have weird DLC setups.
// But it should be fine IIRC, please make an issue if it doesn't.

use log::info;
use serde::Deserialize;
use std::fs::File;
use std::io::Write;
use std::path::Path;

#[derive(Deserialize)]
struct SteamAppDetails {
    #[serde(rename = "dlc")]
    dlc_ids: Option<Vec<u32>>,
    name: Option<String>,
}

#[derive(Deserialize)]
struct SteamAppResponse {
    success: bool,
    data: Option<SteamAppDetails>,
}

#[derive(Deserialize)]
struct SteamStoreResponse {
    #[serde(flatten)]
    apps: std::collections::HashMap<String, SteamAppResponse>,
}

pub async fn fetch_and_write_dlcs(app_id: &str, steam_settings_dir: &Path) -> Result<(), String> {
    info!("Fetching DLCs for AppID: {}", app_id);

    // Fetch main app details from Steam Store API.
    let url = format!(
        "https://store.steampowered.com/api/appdetails?appids={}&l=english",
        app_id
    );
    let response = reqwest::get(&url)
        .await
        .map_err(|e| format!("Failed to fetch DLCs for AppID {}: {}", app_id, e))?;

    let json: SteamStoreResponse = response
        .json()
        .await
        .map_err(|e| format!("Failed to parse DLC response for AppID {}: {}", app_id, e))?;

    let mut dlc_lines = vec!["[app::dlcs]\nunlock_all=0\n".to_string()];
    let mut dlc_count = 0;

    // Extract DLC IDs and names
    if let Some(app_response) = json.apps.get(app_id) {
        if app_response.success {
            if let Some(data) = &app_response.data {
                if let Some(dlc_ids) = &data.dlc_ids {
                    for &dlc_id in dlc_ids {
                        let dlc_url = format!(
                            "https://store.steampowered.com/api/appdetails?appids={}",
                            dlc_id
                        );
                        let dlc_response = reqwest::get(&dlc_url).await.map_err(|e| {
                            format!("Failed to fetch DLC {} details: {}", dlc_id, e)
                        })?;
                        let dlc_json: SteamStoreResponse =
                            dlc_response.json().await.map_err(|e| {
                                format!("Failed to parse DLC {} response: {}", dlc_id, e)
                            })?;

                        if let Some(dlc_response) = dlc_json.apps.get(&dlc_id.to_string()) {
                            if dlc_response.success {
                                if let Some(dlc_data) = &dlc_response.data {
                                    // Only include DLC if it has a valid name, otherwise skip it.
                                    if let Some(dlc_name) = &dlc_data.name {
                                        dlc_lines.push(format!("{}={}\n", dlc_id, dlc_name));
                                        dlc_count += 1;
                                    } else {
                                        info!("Skipping DLC {}: No valid name found", dlc_id);
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    if dlc_count == 0 {
        info!(
            "No DLCs found for AppID {}.",
            app_id
        );
    } else {
        info!("Fetched {} DLCs for AppID {}", dlc_count, app_id);
    }

    // Write configs.app.ini.
    let app_ini = steam_settings_dir.join("configs.app.ini");
    let app_ini_content = dlc_lines.join("");
    let mut file = File::create(&app_ini).map_err(|e| {
        format!(
            "Failed to create configs.app.ini at {}: {}",
            app_ini.display(),
            e
        )
    })?;
    file.write_all(app_ini_content.as_bytes())
        .map_err(|e| format!("Failed to write configs.app.ini: {}", e))?;
    info!("Created configs.app.ini at {}", app_ini.display());

    Ok(())
}