use log::info;
use std::fs;
use std::path::Path;

pub fn handle_unpacked_exe(original_exe_path: &Path) -> Result<(), String> {
    let unpacked_path = original_exe_path.with_file_name(format!(
        "{}.unpacked.exe",
        original_exe_path.file_name().unwrap().to_string_lossy()
    ));

    // If no unpacked file exists, I'll just assume no DRM or unpacking failed, which is normal for Steamless.
    if !unpacked_path.exists() {
        info!(
            "No unpacked file found for {}; no DRM detected or unpacking failed.",
            original_exe_path.display()
        );
        return Ok(());
    }

    let backup_path = original_exe_path.with_extension("svrn");

    // Backup original if not already backed up.
    if !backup_path.exists() {
        fs::rename(original_exe_path, &backup_path)
            .map_err(|e| format!("Failed to backup original EXE to .svrn: {}", e))?;
        info!("Backed up original EXE to: {}", backup_path.display());
    } else {
        // Remove original if backup exists to avoid conflicts.
        fs::remove_file(original_exe_path)
            .map_err(|e| format!("Failed to remove existing original EXE: {}", e))?;
        info!("Removed original EXE (backup already exists).");
    }

    // Replace original with unpacked version.
    fs::rename(&unpacked_path, original_exe_path)
        .map_err(|e| format!("Failed to replace original with unpacked EXE: {}", e))?;
    info!(
        "Replaced original EXE with unpacked version: {}",
        original_exe_path.display()
    );

    Ok(())
}
