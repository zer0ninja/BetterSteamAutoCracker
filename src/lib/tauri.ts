import { open } from "@tauri-apps/plugin-dialog";
import { invoke } from "@tauri-apps/api/core";

/**
 * Interface representing the application settings.
 */
export interface Settings {
  theme: "light" | "dark";
}

/**
 * Interface representing a Steam game.
 */
export interface Game {
  appid: number;
  name: string;
}

/**
 * Retrieves the current settings.
 * @returns A promise that resolves to the current settings.
 * @throws An error if the settings retrieval fails.
 */
export async function getAppSettings(): Promise<Settings> {
  try {
    const settings = await invoke<Settings>("cmd_get_settings");
    return settings;
  } catch (error) {
    console.error("Error loading settings:", error);
    throw new Error(`Failed to load settings: ${error}`);
  }
}

/**
 * Sets the settings and persists them.
 * @param settings The new settings to apply.
 * @throws An error if the settings update fails.
 */
export async function setAppSettings(settings: Settings): Promise<void> {
  try {
    await invoke("cmd_set_settings", { newSettings: settings });
  } catch (error) {
    console.error("Error setting settings:", error);
    throw new Error(`Failed to set settings: ${error}`);
  }
}

/**
 * Opens a directory selection dialog and returns the selected directory path.
 * @returns A promise that resolves to the selected directory path as a string, or null if no directory is selected.
 * @throws An error if the dialog operation fails.
 */
export async function selectDirectory(): Promise<string | null> {
  try {
    const selected = await open({
      multiple: false,
      directory: true,
    });
    return selected as string | null;
  } catch (error) {
    console.error("Error selecting directory:", error);
    throw new Error(`Failed to select directory: ${error}`);
  }
}

/**
 * Checks for DRM using the Tauri `check_drm` command.
 * @param appId - The Steam App ID as a string.
 * @returns A promise that resolves to the DRM status message from the backend.
 * @throws An error if the DRM check fails.
 */
export async function checkDrm(
  appId: string,
  attempt: number = 1
): Promise<string> {
  if (attempt > 3) {
    throw new Error("Maximum DRM check attempts reached.");
  }
  try {
    const result = await invoke<string>("cmd_check_drm", { appId });
    return result;
  } catch (error) {
    throw new Error(`Failed to check DRM: ${error}`);
  }
}

/**
 * Command that applies Steamless and Goldberg Steam Emulator.
 * @param appId - The Steam App ID as a string.
 * @param folderPath - The path to the game directory.
 * @param language - Optional language code for localization (e.g., "english").
 * @returns A promise that resolves to the success message from the backend, or throws an error.
 */
export async function applyCrack(
  appId: string,
  folderPath: string,
  language?: string
): Promise<string> {
  try {
    const drmResult = await checkDrm(appId);
    console.log(`DRM Check Result: ${drmResult}`);

    const result = await invoke<string>("cmd_apply_crack", {
      appId,
      gameLocation: folderPath,
      language,
    });
    return `DRM Check: ${drmResult}\nCrack Result: ${result}`;
  } catch (error) {
    throw new Error(`Failed to apply crack: ${error}`);
  }
}

/**
 * Searches for up to 5 matching games by name.
 * @param title The search term (game title).
 * @returns An array of matched games with name and appid.
 */
export async function searchGame(title: string): Promise<Game[]> {
  try {
    return await invoke<Game[]>("cmd_get_game", { title });
  } catch (error) {
    console.error("Error searching game:", error);
    return [];
  }
}
