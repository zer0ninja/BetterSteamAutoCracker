import { checkDrm } from "../lib/tauri";

/**
 * Checks if a Steam game uses Denuvo DRM by invoking the Tauri `check_drm` command.
 * @param appId - The Steam App ID as a string.
 * @returns A promise that resolves to true if Denuvo is detected, false otherwise.
 */
export async function checkForDRM(appId: string): Promise<boolean> {
  try {
    const result = await checkDrm(appId);
    const hasDenuvo = result.toLowerCase().includes("denuvo");
    console.log(
      `App ID ${appId}: ${
        hasDenuvo ? "Uses Denuvo DRM" : `No Denuvo DRM (${result})`
      }`
    );
    return hasDenuvo;
  } catch (error) {
    console.error(`Failed to check DRM for App ID ${appId}:`, error);
    return false;
  }
}
