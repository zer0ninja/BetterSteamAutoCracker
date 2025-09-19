import { checkDrm } from "../lib/tauri";

/**
 * Checks if a Steam game uses Denuvo DRM by invoking the Tauri `check_drm` command.
 * @param appId - The Steam App ID as a string.
 * @param attempt - The current attempt number (default: 1).
 * @returns A promise that resolves to true if Denuvo is detected, false otherwise.
 * @throws An error if the maximum DRM check attempts are reached.
 */
export async function checkForDRM(
  appId: string,
  attempt: number = 1
): Promise<boolean> {
  if (attempt > 3) {
    throw new Error("Maximum attempts reached.");
  }
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
