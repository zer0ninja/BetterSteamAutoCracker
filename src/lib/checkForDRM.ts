import { fetch } from "@tauri-apps/plugin-http";

export interface SteamAppDetails {
  [appId: string]: {
    success: boolean;
    data?: {
      drm_notice?: string;
    };
  };
}

export async function checkForDRM(appId: string): Promise<boolean> {
  try {
    const response = await fetch(
      `https://store.steampowered.com/api/appdetails?appids=${appId}&l=english`
    );

    const data: SteamAppDetails = await response.json();

    if (!data[appId]?.success || !data[appId]?.data) {
      return false;
    }

    const drmNotice = data[appId].data.drm_notice;

    if (!drmNotice) {
      return false;
    }

    const hasDenuvo = drmNotice.toLowerCase().includes("denuvo");
    console.log(
      `App ID ${appId}: ${
        hasDenuvo
          ? "Uses Denuvo DRM"
          : `DRM notice found ("${drmNotice}"), but no Denuvo`
      }`
    );

    return hasDenuvo;
  } catch (error) {
    console.error(`Failed to check DRM for App ID ${appId}:`, error);
    return false;
  }
}
