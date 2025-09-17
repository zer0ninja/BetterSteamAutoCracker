//! Known issue:
//! It includes `branch` names instead of just numeric IDs for depots,
//! which may or may not work properly, fixing when I can.

use log::info;
use serde::Deserialize;
use serde_json::Value;
use std::fs::File;
use std::io::Write;
use std::path::Path;

#[derive(Deserialize)]
struct DepotManifest {}

#[derive(Deserialize)]
struct AppInfo {
    depots: Option<std::collections::HashMap<String, Value>>,
}

#[derive(Deserialize)]
struct SteamCmdResponse {
    data: std::collections::HashMap<String, AppInfo>,
}

pub async fn fetch_and_write_depots(app_id: &str, steam_settings_dir: &Path) -> Result<(), String> {
    info!("Fetching depots for AppID: {}", app_id);

    // Fetch depot data from steamcmd.net API.
    let url = format!("https://api.steamcmd.net/v1/info/{}", app_id);
    info!("Requesting URL: {}", url);
    let response = reqwest::get(&url)
        .await
        .map_err(|e| format!("Failed to fetch depots for AppID {}: {}", app_id, e))?;

    if !response.status().is_success() {
        return Err(format!(
            "Failed to fetch depots for AppID {}: HTTP status {}",
            app_id,
            response.status()
        ));
    }

    let body = response
        .text()
        .await
        .map_err(|e| format!("Failed to read response body for AppID {}: {}", app_id, e))?;

    let json: SteamCmdResponse = serde_json::from_str(&body).map_err(|e| {
        format!(
            "Failed to parse depots response for AppID {}: {}",
            app_id, e
        )
    })?;

    let mut depot_lines = Vec::new();
    let mut depot_count = 0;

    // Extract depot IDs.
    if let Some(app_info) = json.data.get(app_id) {
        if let Some(depots) = &app_info.depots {
            for depot_id in depots.keys() {
                depot_lines.push(format!("{}\n", depot_id));
                depot_count += 1;
                info!("Found depot ID: {}", depot_id);
            }
        } else {
            info!("No depots found in app_info for AppID {}", app_id);
        }
    } else {
        info!("No app_info found for AppID {} in response", app_id);
    }

    if depot_count == 0 {
        info!("No depots found for AppID {}", app_id);
    } else {
        info!("Fetched {} depots for AppID {}", depot_count, app_id);
    }

    // Write depots.txt.
    let depots_txt = steam_settings_dir.join("depots.txt");
    let depots_content = depot_lines.join("");
    let mut file = File::create(&depots_txt).map_err(|e| {
        format!(
            "Failed to create depots.txt at {}: {}",
            depots_txt.display(),
            e
        )
    })?;
    file.write_all(depots_content.as_bytes())
        .map_err(|e| format!("Failed to write depots.txt: {}", e))?;
    info!(
        "Created depots.txt at {} with {} depots",
        depots_txt.display(),
        depot_count
    );

    Ok(())
}
