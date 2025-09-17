//! Entry point for the BetterSteamAutoCracker Tauri application.
//! Starts up logger, Tauri plugins, and runs the setup process on startup.

// Allowing modules to be public and readable.
pub mod command;
pub mod config;
pub mod goldberg;
pub mod setup;
pub mod steamless;

use env_logger;
use windows::core::PCWSTR;
use windows::Win32::UI::WindowsAndMessaging::{MessageBoxW, MB_ICONERROR, MB_OK};

// Commands
use crate::command::cmd_apply_crack;

// Checks if WebView2 Runtime is installed on Windows.
// The current implementation of check might not be perfect, but it serves the purpose of ensuring
// that the runtime is present. If not, it prompts the user to install it.
// This is crucial for Tauri applications to function correctly.
fn check_webview2() -> Result<(), Box<dyn std::error::Error>> {
    const KEY_PATH: &str =
        r"SOFTWARE\WOW6432Node\Microsoft\Windows\CurrentVersion\Uninstall\Microsoft EdgeWebView";

    let hkml = winreg::RegKey::predef(winreg::enums::HKEY_LOCAL_MACHINE);
    let key = hkml
        .open_subkey(KEY_PATH)
        .map_err(|e| format!("Failed to open registry key: {}", e))?;
    let version: String = key
        .get_value("Version")
        .map_err(|e| format!("Failed to read registry value: {}", e))?;

    if version.is_empty() {
        let text: Vec<u16> = "WebView2 Runtime is missing. Click OK to download it from the official Microsoft site."
            .encode_utf16()
            .chain(std::iter::once(0))
            .collect();
        let caption: Vec<u16> = "Error".encode_utf16().chain(std::iter::once(0)).collect();

        unsafe {
            MessageBoxW(
                None,
                PCWSTR(text.as_ptr()),
                PCWSTR(caption.as_ptr()),
                MB_ICONERROR | MB_OK,
            );
        }

        webbrowser::open("https://go.microsoft.com/fwlink/p/?LinkId=2124703")?;
        std::process::exit(1);
    }
    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    env_logger::init();
    if cfg!(target_os = "windows") {
        if let Err(e) = check_webview2() {
            eprintln!("WebView2 check failed: {}", e);
            std::process::exit(1);
        }
    }

    tauri::Builder::default()
        .plugin(tauri_plugin_updater::Builder::new().build())
        .plugin(tauri_plugin_http::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![cmd_apply_crack])
        .setup(|app| {
            if let Err(e) = tauri::async_runtime::block_on(async {
                crate::setup::setup(app.handle().clone()).await
            }) {
                eprintln!("Setup failed: {}", e);
                std::process::exit(1);
            }
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("An error occurred while trying to open the Tauri application");
}
