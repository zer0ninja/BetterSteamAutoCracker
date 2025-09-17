use crate::steamless::config::CREATE_NO_WINDOW;
use log::info;
use std::os::windows::process::CommandExt;
use std::path::Path;
use std::process::{Command, Stdio};
use std::thread;
use std::time::Duration;

pub async fn run_steamless_on_exe(
    steamless_cli_path: &Path,
    exe_path: &Path,
) -> Result<(), String> {
    info!("Executing Steamless CLI on: {}", exe_path.display());

    // --quiet: Suppress console output.
    // --keep-bind-section: Preserve import bindings for compatibility.
    // --unpacked-name: Generate .unpacked.exe output file.
    // --all-plugins: Use all available unpacker plugins.
    let status = Command::new(steamless_cli_path)
        .args([
            "--quiet",
            "--keep-bind-section",
            "--unpacked-name",
            "--all-plugins",
            exe_path.to_str().ok_or("Invalid UTF-8 in EXE path")?,
        ])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .creation_flags(CREATE_NO_WINDOW)
        .status()
        .map_err(|e| format!("Failed to execute Steamless CLI: {}", e))?;

    if !status.success() {
        return Err(format!(
            "Steamless CLI failed with exit code: {}",
            status.code().unwrap_or(-1)
        ));
    }

    thread::sleep(Duration::from_millis(100));

    Ok(())
}
