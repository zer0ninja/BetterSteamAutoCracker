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

impl Default for Settings {
    fn default() -> Self {
        Settings {
            theme: Theme::Light,
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
