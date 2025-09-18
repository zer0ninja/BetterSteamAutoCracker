use env_logger;
use std::sync::Mutex;
use tauri::{async_runtime, generate_context, generate_handler, Builder, Emitter};

pub mod command;
pub mod config;
pub mod error;
pub mod goldberg;
pub mod settings;
pub mod setup;
pub mod steamless;

use crate::command::{
    cmd_apply_crack, cmd_check_drm, cmd_get_settings, cmd_get_windows_theme, cmd_set_settings,
};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    env_logger::init();

    let settings = settings::load_settings();
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
