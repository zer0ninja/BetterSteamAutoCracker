use env_logger;
use tauri::{Emitter, Builder, async_runtime, generate_handler, generate_context};

pub mod command;
pub mod config;
pub mod error;
pub mod goldberg;
pub mod setup;
pub mod steamless;

use crate::command::cmd_apply_crack;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    env_logger::init();

    Builder::default()
        .plugin(tauri_plugin_updater::Builder::new().build())
        .plugin(tauri_plugin_http::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(generate_handler![cmd_apply_crack])
        .setup(|app| {
            let app_handle = app.handle().clone();
            async_runtime::spawn(async move {
                if let Err(e) = setup::setup(app_handle.clone()).await {
                    eprintln!("Setup failed: {}", e);
                    if let Err(emit_err) = app_handle.emit("setup-error", format!("Setup failed: {}", e)) {
                        eprintln!("Failed to emit error: {}", emit_err);
                    }
                    std::process::exit(1);
                }
            });
            Ok(())
        })
        .run(generate_context!())
        .expect("An error occurred while trying to run Better Steam Auto Cracker.");
}