// Taken from Mr.Goldberg's original Steam Emulator project,
// and translated to Rust. Generates steam_interfaces.txt

use log::info;
use regex::Regex;
use std::fs::File;
use std::io::{Read, Write};
use std::path::Path;

const INTERFACE_PATTERNS: &[&str] = &[
    r"STEAMAPPS_INTERFACE_VERSION\d+",
    r"STEAMAPPLIST_INTERFACE_VERSION\d+",
    r"STEAMAPPTICKET_INTERFACE_VERSION\d+",
    r"SteamClient\d+",
    r"STEAMCONTROLLER_INTERFACE_VERSION",
    r"SteamController\d+",
    r"SteamFriends\d+",
    r"SteamGameServerStats\d+",
    r"SteamGameCoordinator\d+",
    r"SteamGameServer\d+",
    r"STEAMHTMLSURFACE_INTERFACE_VERSION_\d+",
    r"STEAMHTTP_INTERFACE_VERSION\d+",
    r"SteamInput\d+",
    r"STEAMINVENTORY_INTERFACE_V\d+",
    r"SteamMatchMakingServers\d+",
    r"SteamMatchMaking\d+",
    r"SteamMatchGameSearch\d+",
    r"SteamParties\d+",
    r"STEAMMUSIC_INTERFACE_VERSION\d+",
    r"STEAMMUSICREMOTE_INTERFACE_VERSION\d+",
    r"SteamNetworkingMessages\d+",
    r"SteamNetworkingSockets\d+",
    r"SteamNetworkingUtils\d+",
    r"SteamNetworking\d+",
    r"STEAMPARENTALSETTINGS_INTERFACE_VERSION\d+",
    r"STEAMREMOTEPLAY_INTERFACE_VERSION\d+",
    r"STEAMREMOTESTORAGE_INTERFACE_VERSION\d+",
    r"STEAMSCREENSHOTS_INTERFACE_VERSION\d+",
    r"STEAMTIMELINE_INTERFACE_V\d+",
    r"STEAMUGC_INTERFACE_VERSION\d+",
    r"SteamUser\d+",
    r"STEAMUSERSTATS_INTERFACE_VERSION\d+",
    r"SteamUtils\d+",
    r"STEAMVIDEO_INTERFACE_V\d+",
    r"STEAMUNIFIEDMESSAGES_INTERFACE_VERSION\d+",
    r"SteamMasterServerUpdater\d+",
];

pub fn generate_steam_interfaces(
    original_dll_path: &Path,
    steam_settings_dir: &Path,
) -> Result<u32, String> {
    info!(
        "Generating steam_interfaces.txt from: {}",
        original_dll_path.display()
    );

    if !original_dll_path.exists() {
        return Err(format!(
            "Original DLL not found: {}",
            original_dll_path.display()
        ));
    }

    let mut dll_file = File::open(original_dll_path)
        .map_err(|e| format!("Failed to open DLL {}: {}", original_dll_path.display(), e))?;
    let mut dll_contents = Vec::new();
    dll_file
        .read_to_end(&mut dll_contents)
        .map_err(|e| format!("Failed to read DLL {}: {}", original_dll_path.display(), e))?;

    let dll_contents_str = String::from_utf8_lossy(&dll_contents);
    if dll_contents_str.is_empty() {
        return Err("DLL content is empty or unreadable. Is the DLL valid?".to_string());
    }

    let interfaces_file = steam_settings_dir.join("steam_interfaces.txt");
    let mut out_file = File::create(&interfaces_file).map_err(|e| {
        format!(
            "Failed to create steam_interfaces.txt at {}: {}",
            interfaces_file.display(),
            e
        )
    })?;

    let mut total_matches = 0u32;

    for pattern in INTERFACE_PATTERNS {
        match Regex::new(pattern) {
            Ok(re) => {
                for mat in re.find_iter(&dll_contents_str) {
                    writeln!(out_file, "{}", mat.as_str())
                        .map_err(|e| format!("Failed to write to steam_interfaces.txt: {}", e))?;
                    total_matches += 1;
                }
            }
            Err(e) => {
                info!("Invalid regex pattern '{}': {}", pattern, e);
            }
        }
    }

    out_file
        .sync_all()
        .map_err(|e| format!("Failed to sync steam_interfaces.txt: {}", e))?;

    info!(
        "Generated steam_interfaces.txt with {} matches at {}",
        total_matches,
        interfaces_file.display()
    );
    Ok(total_matches)
}
