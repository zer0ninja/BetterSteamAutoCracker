use tauri::command;

#[cfg(target_os = "windows")]
#[command]
pub fn get_windows_theme() -> Result<String, String> {
    use winreg::enums::*;
    use winreg::RegKey;

    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let personalize = hkcu.open_subkey("Software\\Microsoft\\Windows\\CurrentVersion\\Themes\\Personalize")
        .map_err(|e| format!("Failed to open registry key: {:?}", e))?;
    let theme: u32 = personalize.get_value("AppsUseLightTheme")
        .map_err(|e| format!("Failed to get AppsUseLightTheme: {:?}", e))?;
    Ok(if theme == 0 { "dark".into() } else { "light".into() })
}
