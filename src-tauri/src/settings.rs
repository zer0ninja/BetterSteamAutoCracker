use dirs::data_dir;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use winreg::enums::*;
use winreg::RegKey;

use crate::config::FOLDER;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum Theme {
    #[serde(rename = "light")]
    Light,
    #[serde(rename = "dark")]
    Dark,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Settings {
    pub theme: Theme,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AppData {
    pub passed_messageboxw: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Settings {
            theme: Theme::Light,
        }
    }
}

impl Default for AppData {
    fn default() -> Self {
        AppData {
            passed_messageboxw: false,
        }
    }
}

pub fn get_system_theme() -> Theme {
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    match hkcu.open_subkey("Software\\Microsoft\\Windows\\CurrentVersion\\Themes\\Personalize") {
        Ok(personalize) => match personalize.get_value::<u32, _>("AppsUseLightTheme") {
            Ok(theme) => {
                if theme == 0 {
                    Theme::Dark
                } else {
                    Theme::Light
                }
            }
            Err(e) => {
                eprintln!("Failed to get AppsUseLightTheme: {}", e);
                Theme::Light
            }
        },
        Err(e) => {
            eprintln!("Failed to open registry key: {}", e);
            Theme::Light
        }
    }
}

pub fn load_settings() -> Settings {
    let settings_path = data_dir()
        .map(|dir| dir.join(FOLDER).join("settings").join("theme.json"))
        .unwrap_or_else(|| PathBuf::from("settings.json"));

    if settings_path.exists() {
        match fs::read_to_string(&settings_path) {
            Ok(data) => match serde_json::from_str::<Settings>(&data) {
                Ok(settings) => settings,
                Err(e) => {
                    eprintln!("Failed to parse settings: {}, using system theme", e);
                    Settings {
                        theme: get_system_theme(),
                    }
                }
            },
            Err(e) => {
                eprintln!(
                    "Failed to read settings from {}: {}, using system theme",
                    settings_path.display(),
                    e
                );
                Settings {
                    theme: get_system_theme(),
                }
            }
        }
    } else {
        Settings {
            theme: get_system_theme(),
        }
    }
}

pub fn load_app_data() -> AppData {
    let data_path = data_dir()
        .map(|dir| dir.join(FOLDER).join("settings").join("data.json"))
        .unwrap_or_else(|| PathBuf::from("data.json"));

    if data_path.exists() {
        match fs::read_to_string(&data_path) {
            Ok(data) => match serde_json::from_str::<AppData>(&data) {
                Ok(app_data) => app_data,
                Err(e) => {
                    eprintln!("Failed to parse app data: {}, using default", e);
                    AppData::default()
                }
            },
            Err(e) => {
                eprintln!(
                    "Failed to read app data from {}: {}, using default",
                    data_path.display(),
                    e
                );
                AppData::default()
            }
        }
    } else {
        AppData::default()
    }
}

pub fn save_app_data(app_data: &AppData) -> Result<(), Box<dyn std::error::Error>> {
    let data_path = data_dir()
        .map(|dir| dir.join(FOLDER).join("settings").join("data.json"))
        .unwrap_or_else(|| PathBuf::from("data.json"));

    if let Some(parent) = data_path.parent() {
        fs::create_dir_all(parent)?;
    }

    let data = serde_json::to_string_pretty(app_data)?;
    fs::write(&data_path, data)?;
    Ok(())
}