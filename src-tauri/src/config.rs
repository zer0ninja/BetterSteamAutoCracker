// URL for S3 bucket hosting files.
// These include:
// - steam_api(64).dll and steamclient(64).dll (Goldberg Steam Emulator files for emulating Steam's API).
// - Steamless.v3.1.0.5.-.by.atom0s.zip (Removes Steam DRM from game executables).
//
// Note: You are free to modify this URL to point to an alternative mirror if needed, but you'll have
// to modify setup.rs as well.
pub const S3: &str = "https://s3.lillianne.solutions/";

// Official GitHub repository URL for the project.
// Used for update checks and update prompts.
// Important: When creating a fork, do not modify this constant without
// explicit permission from "0xSovereign" to avoid breaking attribution.
pub const GITHUB: &str = "https://github.com/0xSovereign/BetterSteamAutoCracker";
pub const GITHUB_API: &str =
    "https://api.github.com/repos/0xSovereign/BetterSteamAutoCracker/releases/latest"; // For update checks.

// Current application version, automatically set from Cargo.toml during build.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

// Steam Web API key for getting achievements.
// Get the key key from: https://steamcommunity.com/dev/apikey
pub const STEAM_API_KEY: &str = ""; // Needed

// Application folder name for storing cache data.
pub const FOLDER: &str = "sovereign.bsac.app"; // It has to match tauri.conf.json's identifier.
