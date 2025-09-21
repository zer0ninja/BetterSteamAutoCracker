use tauri::{async_runtime, generate_context, generate_handler, Builder, Emitter};

use env_logger;
use std::sync::Mutex;

pub mod command;
pub mod config;
pub mod dialog;
pub mod error;
pub mod goldberg;
pub mod settings;
pub mod setup;
pub mod steamless;

use crate::command::{
    cmd_apply_crack, cmd_check_drm, cmd_get_game, cmd_get_settings, cmd_get_windows_theme, cmd_set_settings,
};
use crate::dialog::{show_webview2_dialog, show_foss_dialog};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    env_logger::init();

    // Check for WebView2 installation
    let is_webview2_installed = std::path::Path::new("C:\\Program Files (x86)\\Microsoft\\EdgeWebView\\Application").exists();

    if !is_webview2_installed {
        show_webview2_dialog();
        std::process::exit(1);
    }

    let settings = settings::load_settings();
    let app_data = settings::load_app_data();

    if !app_data.passed_messageboxw {
        let result = show_foss_dialog();

        // If the user clicks OK (result == 1), update app_data
        if result == 1 {
            let mut new_app_data = app_data.clone();
            new_app_data.passed_messageboxw = true;
            if let Err(e) = settings::save_app_data(&new_app_data) {
                eprintln!("Failed to save app data: {}", e);
            }
        } else {
            eprintln!("MessageBoxW did not return OK, exiting.");
            std::process::exit(1);
        }
    }

    Builder::default()
        .manage(Mutex::new(settings))
        .plugin(tauri_plugin_updater::Builder::new().build())
        .plugin(tauri_plugin_http::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(generate_handler![
            cmd_apply_crack,
            cmd_check_drm,
            cmd_get_windows_theme,
            cmd_get_settings,
            cmd_set_settings,
            cmd_get_game,
        ])
        .setup(|app| {
            let app_handle = app.handle().clone();
            async_runtime::spawn({
                let app_handle = app_handle.clone();
                async move {
                    if let Err(e) = setup::setup(app_handle.clone()).await {
                        eprintln!("Setup failed: {}", e);
                        if let Err(emit_err) =
                            app_handle.emit("setup-error", format!("Setup failed: {}", e))
                        {
                            eprintln!("Failed to emit error: {}", emit_err);
                        }
                        std::process::exit(1);
                    }
                }
            });

            Ok(())
        })
        .run(generate_context!())
        .expect("An error occurred while running Better Steam Auto Cracker.");
}