import { open } from "@tauri-apps/plugin-dialog";
import { invoke } from "@tauri-apps/api/core";

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
 * Invokes the `cmd_apply_crack` Tauri command to apply Steamless DRM removal and Goldberg Steam Emulator.
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
    const result = await invoke<string>("cmd_apply_crack", {
      appId,
      gameLocation: folderPath,
      language,
    });
    return result;
  } catch (error) {
    throw new Error(`Failed to apply crack: ${error}`);
  }
}
