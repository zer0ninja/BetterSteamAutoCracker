use log::info;
use serde::Deserialize;
use std::fs::File;
use std::io::Write;
use std::path::Path;

#[derive(Deserialize)]
struct AppInfo {
    common: Option<Common>,
}

#[derive(Deserialize)]
struct Common {
    languages: Option<std::collections::HashMap<String, String>>,
}

#[derive(Deserialize)]
struct SteamCmdResponse {
    data: std::collections::HashMap<String, AppInfo>,
}

pub async fn fetch_and_write_languages(
    app_id: &str,
    steam_settings_dir: &Path,
) -> Result<(), String> {
    info!("Fetching supported languages for AppID: {}", app_id);

    let url = format!("https://api.steamcmd.net/v1/info/{}", app_id);
    info!("Requesting URL: {}", url);
    let response = reqwest::get(&url)
        .await
        .map_err(|e| format!("Failed to fetch languages for AppID {}: {}", app_id, e))?;

    if !response.status().is_success() {
        return Err(format!(
            "Failed to fetch languages for AppID {}: HTTP status {}",
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
            "Failed to parse languages response for AppID {}: {}",
            app_id, e
        )
    })?;

    let mut language_lines = Vec::new();
    let mut language_count = 0;

    // Extract supported languages
    if let Some(app_info) = json.data.get(app_id) {
        if let Some(common) = &app_info.common {
            if let Some(languages) = &common.languages {
                for (language, support) in languages {
                    if support == "1" || support.to_lowercase() == "true" {
                        language_lines.push(format!("{}\n", language));
                        language_count += 1;
                        info!("Found supported language: {}", language);
                    }
                }
            } else {
                info!("No languages found in common for AppID {}", app_id);
            }
        } else {
            info!("No common field found in app_info for AppID {}", app_id);
        }
    } else {
        info!("No app_info found for AppID {} in response", app_id);
    }

    if language_count == 0 {
        info!(
            "No supported languages found for AppID {}, defaulting to english",
            app_id
        );
        language_lines.push("english\n".to_string());
        language_count = 1;
    }

    info!(
        "Fetched {} supported languages for AppID {}",
        language_count, app_id
    );

    // Write supported_languages.txt.
    let languages_txt = steam_settings_dir.join("supported_languages.txt");
    let languages_content = language_lines.join("");
    let mut file = File::create(&languages_txt).map_err(|e| {
        format!(
            "Failed to create supported_languages.txt at {}: {}",
            languages_txt.display(),
            e
        )
    })?;
    file.write_all(languages_content.as_bytes())
        .map_err(|e| format!("Failed to write supported_languages.txt: {}", e))?;
    info!(
        "Created supported_languages.txt at {} with {} languages",
        languages_txt.display(),
        language_count
    );

    Ok(())
}
