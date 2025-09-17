use crate::config::FOLDER;
use crate::goldberg::{depots, dlcs, languages};
use dirs::data_dir;
use log::info;
use serde_json::json;
use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::thread;
use std::time::Duration;
use tauri::{AppHandle, Emitter};
use walkdir::WalkDir;
use zip::{write::FileOptions, ZipWriter};

// I had to do this function as sometimes it just fails due to certain file locks
// This implementation is a complete piece of shit, and I'd appreciate pull request improvements
fn copy_with_retry(src: &Path, dst: &Path, max_retries: u32) -> Result<(), String> {
    for attempt in 0..=max_retries {
        match fs::copy(src, dst) {
            Ok(_) => {
                info!(
                    "Copied {} to {} on attempt {}",
                    src.display(),
                    dst.display(),
                    attempt + 1
                );
                return Ok(());
            }
            Err(e) => {
                if attempt == max_retries {
                    return Err(format!(
                        "Failed to copy {} to {} after {} retries: {}",
                        src.display(), dst.display(), max_retries + 1, e
                    ));
                }
                info!(
                    "Copy attempt {} failed for {} to {}: {}. Retrying in 1s",
                    attempt + 1,
                    src.display(),
                    dst.display(),
                    e
                );
                thread::sleep(Duration::from_secs(1));
            }
        }
    }
    Ok(())
}

pub async fn apply_goldberg(
    app_handle: AppHandle,
    game_location: String,
    app_id: String,
    language: Option<String>,
) -> Result<String, String> {
    let cache_dir = data_dir()
        .ok_or("Failed to get app data directory")?
        .join(FOLDER)
        .to_string_lossy()
        .into_owned();

    let game_path = PathBuf::from(&game_location);
    if !game_path.is_dir() {
        return Err(format!("Invalid game directory: {}", game_location));
    }

    // Find Steam API DLLs
    let mut found_dlls = Vec::new();
    for entry in WalkDir::new(&game_path).into_iter().filter_map(Result::ok) {
        let path = entry.path();
        if path.is_file() {
            let file_name = path.file_name().unwrap_or_default().to_string_lossy();
            if file_name.eq_ignore_ascii_case("steam_api64.dll") {
                found_dlls.push((path.to_path_buf(), true));
                info!("Found steam_api64.dll at: {}", path.display());
            } else if file_name.eq_ignore_ascii_case("steam_api.dll") {
                found_dlls.push((path.to_path_buf(), false));
                info!("Found steam_api.dll at: {}", path.display());
            }
        }
    }

    if found_dlls.is_empty() {
        return Err("No Steam API DLLs found in game directory".to_string());
    }

    // Find largest executable for steamclient(64).dll placement
    let mut largest_exe: Option<PathBuf> = None;
    let mut max_size = 0;
    for entry in WalkDir::new(&game_path).into_iter().filter_map(Result::ok) {
        let path = entry.path();
        if path.is_file()
            && path
                .extension()
                .map_or(false, |ext| ext.to_ascii_lowercase() == "exe")
        {
            if let Ok(metadata) = fs::metadata(path) {
                let size = metadata.len();
                if size > max_size {
                    max_size = size;
                    largest_exe = Some(path.to_path_buf());
                }
            }
        }
    }

    let dll_count = found_dlls.len() as f32;
    let dll_step = 50.0 / dll_count;
    let mut current_progress = 50.0;

    let substeps_per_dll = 16.0;
    let substep = dll_step / substeps_per_dll;

    let mut success_messages = Vec::new();

    for (dll_path, is_64bit) in found_dlls {
        app_handle.emit("crack-progress", &json!({"progress": current_progress as u32, "message": format!("Processing DLL: {}", dll_path.display())})).map_err(|e| format!("Failed to emit progress: {}", e))?;

        let dll_name = if is_64bit {
            "steam_api64.dll"
        } else {
            "steam_api.dll"
        };
        let client_name = if is_64bit {
            "steamclient64.dll"
        } else {
            "steamclient.dll"
        };
        let source_dll_path = Path::new(&cache_dir).join(dll_name);
        let source_client_path = Path::new(&cache_dir).join(client_name);
        let source_sound_path = Path::new(&cache_dir).join("overlay_achievement_notification.wav");
        let source_font_path = Path::new(&cache_dir).join("Roboto-Medium.ttf");

        if !source_dll_path.exists() {
            return Err(format!(
                "{} not found in cache: {}",
                dll_name,
                source_dll_path.display()
            ));
        }
        if !source_client_path.exists() {
            return Err(format!(
                "{} not found in cache: {}",
                client_name,
                source_client_path.display()
            ));
        }
        if !source_sound_path.exists() {
            return Err(format!(
                "overlay_achievement_notification.wav not found in cache: {}",
                source_sound_path.display()
            ));
        }
        if !source_font_path.exists() {
            return Err(format!(
                "Roboto-Medium.ttf not found in cache: {}",
                source_font_path.display()
            ));
        }
        current_progress += substep;
        app_handle
            .emit(
                "crack-progress",
                &json!({"progress": current_progress as u32, "message": "Validated source files"}),
            )
            .map_err(|e| format!("Failed to emit progress: {}", e))?;

        // Create steam_settings directory
        let dll_dir = dll_path
            .parent()
            .ok_or("Failed to get DLL parent directory")?;
        let steam_settings_dir = dll_dir.join("steam_settings");
        fs::create_dir_all(&steam_settings_dir).map_err(|e| {
            format!(
                "Failed to create steam_settings directory {}: {}",
                steam_settings_dir.display(),
                e
            )
        })?;
        info!(
            "Created steam_settings directory: {}",
            steam_settings_dir.display()
        );
        current_progress += substep;
        app_handle.emit("crack-progress", &json!({"progress": current_progress as u32, "message": "Created steam_settings directory"})).map_err(|e| format!("Failed to emit progress: {}", e))?;

        // Generate steam_interfaces.txt
        let _ = super::interfaces::generate_steam_interfaces(&dll_path, &steam_settings_dir)
            .map_err(|e| format!("Failed to generate steam_interfaces.txt: {}", e))?;
        current_progress += substep;
        app_handle.emit("crack-progress", &json!({"progress": current_progress as u32, "message": "Generated steam_interfaces.txt"})).map_err(|e| format!("Failed to emit progress: {}", e))?;

        // Fetch achievements, stats, and images
        super::achievements::fetch_achievements(
            &app_handle,
            &app_id,
            language.clone(),
            &steam_settings_dir,
            dll_step,
            substeps_per_dll,
        )
        .await
        .map_err(|e| format!("Failed to fetch achievements: {}", e))?;
        current_progress += substep;
        app_handle.emit("crack-progress", &json!({"progress": current_progress as u32, "message": "Fetched achievements, stats, and images"})).map_err(|e| format!("Failed to emit progress: {}", e))?;

        // Copy Goldberg DLLs
        let target_dll_path = dll_dir.join(dll_name);
        let backup_dir = dll_dir.join("Crack");
        fs::create_dir_all(&backup_dir).map_err(|e| {
            format!(
                "Failed to create backup directory {}: {}",
                backup_dir.display(),
                e
            )
        })?;
        current_progress += substep;
        app_handle.emit("crack-progress", &json!({"progress": current_progress as u32, "message": "Created backup directory"})).map_err(|e| format!("Failed to emit progress: {}", e))?;

        // Creae .svrn backup in dll_dir if original exists and is not already Goldberg
        if target_dll_path.exists() {
            if let (Ok(meta_target), Ok(meta_source)) = (
                fs::metadata(&target_dll_path),
                fs::metadata(&source_dll_path),
            ) {
                if meta_target.len() != meta_source.len() {
                    let svrn_path = dll_dir.join(format!(
                        "{}.svrn",
                        dll_name.strip_suffix(".dll").unwrap_or(dll_name)
                    ));
                    if svrn_path.exists() {
                        fs::remove_file(&svrn_path).ok();
                    }
                    copy_with_retry(&target_dll_path, &svrn_path, 5)?;
                    info!(
                        "Backed up original {} as {}.svrn",
                        dll_name,
                        svrn_path.display()
                    );
                } else {
                    info!(
                        "Existing {} matches Goldberg size, skipping .svrn backup (already cracked)",
                        dll_name
                    );
                }
            } else {
                let svrn_path = dll_dir.join(format!(
                    "{}.svrn",
                    dll_name.strip_suffix(".dll").unwrap_or(dll_name)
                ));
                if svrn_path.exists() {
                    fs::remove_file(&svrn_path).ok();
                }
                copy_with_retry(&target_dll_path, &svrn_path, 5)?;
                info!(
                    "Backed up {} as {}.svrn (metadata check failed)",
                    dll_name,
                    svrn_path.display()
                );
            }
        }
        current_progress += substep;
        app_handle
            .emit(
                "crack-progress",
                &json!({"progress": current_progress as u32, "message": "Backed up original DLL"}),
            )
            .map_err(|e| format!("Failed to emit progress: {}", e))?;

        copy_with_retry(&source_dll_path, &target_dll_path, 5)?;
        info!("Copied {} to {}", dll_name, target_dll_path.display());
        current_progress += substep;
        app_handle
            .emit(
                "crack-progress",
                &json!({"progress": current_progress as u32, "message": "Copied Goldberg DLL"}),
            )
            .map_err(|e| format!("Failed to emit progress: {}", e))?;

        // Copy steamclient(64).dll next to largest executable
        let mut target_client_path = None;
        if let Some(exe_path) = &largest_exe {
            let exe_dir = exe_path
                .parent()
                .ok_or("Failed to get executable directory")?;
            let client_path = exe_dir.join(client_name);
            copy_with_retry(&source_client_path, &client_path, 5)?;
            info!("Copied {} to {}", client_name, client_path.display());
            target_client_path = Some(client_path);
        }
        current_progress += substep;
        app_handle
            .emit(
                "crack-progress",
                &json!({"progress": current_progress as u32, "message": "Copied steamclient DLL"}),
            )
            .map_err(|e| format!("Failed to emit progress: {}", e))?;

        // Copy sound and font
        let target_sound_path = steam_settings_dir
            .join("sounds")
            .join("overlay_achievement_notification.wav");
        fs::create_dir_all(target_sound_path.parent().unwrap()).map_err(|e| {
            format!(
                "Failed to create sounds directory {}: {}",
                target_sound_path.parent().unwrap().display(),
                e
            )
        })?;
        fs::copy(&source_sound_path, &target_sound_path).map_err(|e| {
            format!(
                "Failed to copy overlay sound to {}: {}",
                target_sound_path.display(),
                e
            )
        })?;
        info!("Copied overlay sound to {}", target_sound_path.display());

        let target_font_path = steam_settings_dir.join("fonts").join("Roboto-Medium.ttf");
        fs::create_dir_all(target_font_path.parent().unwrap()).map_err(|e| {
            format!(
                "Failed to create fonts directory {}: {}",
                target_font_path.parent().unwrap().display(),
                e
            )
        })?;
        fs::copy(&source_font_path, &target_font_path).map_err(|e| {
            format!(
                "Failed to copy font to {}: {}",
                target_font_path.display(),
                e
            )
        })?;
        info!("Copied font to {}", target_font_path.display());
        current_progress += substep;
        app_handle.emit("crack-progress", &json!({"progress": current_progress as u32, "message": "Copied sound and font files"})).map_err(|e| format!("Failed to emit progress: {}", e))?;

        // Generate configuration files
        let main_ini = steam_settings_dir.join("configs.main.ini");
        let main_ini_content = "[main::stats]\nrecord_playtime=1\n";
        let mut file = File::create(&main_ini).map_err(|e| {
            format!(
                "Failed to create configs.main.ini at {}: {}",
                main_ini.display(),
                e
            )
        })?;
        file.write_all(main_ini_content.as_bytes())
            .map_err(|e| format!("Failed to write configs.main.ini: {}", e))?;
        info!("Created configs.main.ini at {}", main_ini.display());

        let user_ini = steam_settings_dir.join("configs.user.ini");
        let user_ini_content = format!(
            "[user::general]\naccount_name=Player\naccount_steamid=76561197960287930\nlanguage={}\n",
            language.clone().unwrap_or_else(|| "english".to_string())
        );
        let mut file = File::create(&user_ini).map_err(|e| {
            format!(
                "Failed to create configs.user.ini at {}: {}",
                user_ini.display(),
                e
            )
        })?;
        file.write_all(user_ini_content.as_bytes())
            .map_err(|e| format!("Failed to write configs.user.ini: {}", e))?;
        info!("Created configs.user.ini at {}", user_ini.display());

        let overlay_ini = steam_settings_dir.join("configs.overlay.ini");
        let overlay_ini_content = "[overlay::general]\nenable_experimental_overlay=1\n[overlay::appearance]\nFont_Override=Roboto-Medium.ttf\n";
        let mut file = File::create(&overlay_ini).map_err(|e| {
            format!(
                "Failed to create configs.overlay.ini at {}: {}",
                overlay_ini.display(),
                e
            )
        })?;
        file.write_all(overlay_ini_content.as_bytes())
            .map_err(|e| format!("Failed to write configs.overlay.ini: {}", e))?;
        info!("Created configs.overlay.ini at {}", overlay_ini.display());
        current_progress += substep;
        app_handle.emit("crack-progress", &json!({"progress": current_progress as u32, "message": "Generated configuration files"})).map_err(|e| format!("Failed to emit progress: {}", e))?;

        // Fetch and write DLCs
        dlcs::fetch_and_write_dlcs(&app_id, &steam_settings_dir)
            .await
            .map_err(|e| format!("Failed to fetch and write DLCs: {}", e))?;
        current_progress += substep;
        app_handle
            .emit(
                "crack-progress",
                &json!({"progress": current_progress as u32, "message": "Fetched and wrote DLCs"}),
            )
            .map_err(|e| format!("Failed to emit progress: {}", e))?;

        // Fetch and write depots
        depots::fetch_and_write_depots(&app_id, &steam_settings_dir)
            .await
            .map_err(|e| format!("Failed to fetch and write depots: {}", e))?;
        current_progress += substep;
        app_handle.emit("crack-progress", &json!({"progress": current_progress as u32, "message": "Fetched and wrote depots"})).map_err(|e| format!("Failed to emit progress: {}", e))?;

        // Fetch and write supported languages
        languages::fetch_and_write_languages(&app_id, &steam_settings_dir)
            .await
            .map_err(|e| format!("Failed to fetch and write languages: {}", e))?;
        current_progress += substep;
        app_handle.emit("crack-progress", &json!({"progress": current_progress as u32, "message": "Fetched and wrote languages"})).map_err(|e| format!("Failed to emit progress: {}", e))?;

        // Create steam_appid.txt
        let appid_txt = steam_settings_dir.join("steam_appid.txt");
        let appid_content = format!("{}\n", app_id);
        let mut file = File::create(&appid_txt).map_err(|e| {
            format!(
                "Failed to create steam_appid.txt at {}: {}",
                appid_txt.display(),
                e
            )
        })?;
        file.write_all(appid_content.as_bytes())
            .map_err(|e| format!("Failed to write steam_appid.txt: {}", e))?;
        info!("Created steam_appid.txt at {}", appid_txt.display());
        current_progress += substep;
        app_handle
            .emit(
                "crack-progress",
                &json!({"progress": current_progress as u32, "message": "Created steam_appid.txt"}),
            )
            .map_err(|e| format!("Failed to emit progress: {}", e))?;

        // Create backup
        let archive_path = backup_dir.join("Goldberg.zip");
        let archive_file = File::create(&archive_path)
            .map_err(|e| format!("Failed to create archive {}: {}", archive_path.display(), e))?;
        let mut zip = ZipWriter::new(archive_file);
        let options: FileOptions<'_, ()> =
            FileOptions::default().compression_method(zip::CompressionMethod::Deflated);

        if target_dll_path.exists() {
            let mut file = File::open(&target_dll_path).map_err(|e| {
                format!(
                    "Failed to open {} for archiving: {}",
                    target_dll_path.display(),
                    e
                )
            })?;
            let mut contents = Vec::new();
            file.read_to_end(&mut contents).map_err(|e| {
                format!(
                    "Failed to read {} for archiving: {}",
                    target_dll_path.display(),
                    e
                )
            })?;
            zip.start_file(dll_name, options).map_err(|e| {
                format!(
                    "Failed to start zip entry for {}: {}",
                    target_dll_path.display(),
                    e
                )
            })?;
            zip.write_all(&contents).map_err(|e| {
                format!(
                    "Failed to write {} to archive: {}",
                    target_dll_path.display(),
                    e
                )
            })?;
        }

        if let Some(client_path) = target_client_path {
            if client_path.exists() {
                let mut file = File::open(&client_path).map_err(|e| {
                    format!(
                        "Failed to open {} for archiving: {}",
                        client_path.display(),
                        e
                    )
                })?;
                let mut contents = Vec::new();
                file.read_to_end(&mut contents).map_err(|e| {
                    format!(
                        "Failed to read {} for archiving: {}",
                        client_path.display(),
                        e
                    )
                })?;
                zip.start_file(client_name, options).map_err(|e| {
                    format!(
                        "Failed to start zip entry for {}: {}",
                        client_path.display(),
                        e
                    )
                })?;
                zip.write_all(&contents).map_err(|e| {
                    format!(
                        "Failed to write {} to archive: {}",
                        client_path.display(),
                        e
                    )
                })?;
                info!(
                    "Archived {} to ZIP root (kept in game directory)",
                    client_name
                );
            }
        }

        for entry in WalkDir::new(&steam_settings_dir)
            .into_iter()
            .filter_map(Result::ok)
        {
            let path = entry.path();
            if path.is_file() {
                let relative_path = path.strip_prefix(&steam_settings_dir).map_err(|e| {
                    format!("Failed to get relative path for {}: {}", path.display(), e)
                })?;
                let arc_name = format!("steam_settings/{}", relative_path.to_string_lossy());
                let mut file = File::open(&path).map_err(|e| {
                    format!("Failed to open {} for archiving: {}", path.display(), e)
                })?;
                let mut contents = Vec::new();
                file.read_to_end(&mut contents).map_err(|e| {
                    format!("Failed to read {} for archiving: {}", path.display(), e)
                })?;
                zip.start_file(arc_name, options).map_err(|e| {
                    format!("Failed to start zip entry for {}: {}", path.display(), e)
                })?;
                zip.write_all(&contents)
                    .map_err(|e| format!("Failed to write {} to archive: {}", path.display(), e))?;
            }
        }

        zip.finish().map_err(|e| {
            format!(
                "Failed to finish zip archive {}: {}",
                archive_path.display(),
                e
            )
        })?;
        info!("Created backup archive: {}", archive_path.display());
        current_progress += substep;
        app_handle
            .emit(
                "crack-progress",
                &json!({"progress": current_progress as u32, "message": "Created backup archive"}),
            )
            .map_err(|e| format!("Failed to emit progress: {}", e))?;

        success_messages.push(format!(
            "Applied Goldberg to {} in {}",
            dll_name,
            dll_dir.display()
        ));
    }

    app_handle
        .emit(
            "crack-progress",
            &json!({"progress": 100, "message": "Done"}),
        )
        .map_err(|e| format!("Failed to emit progress: {}", e))?;

    Ok(success_messages.join("\n"))
}
