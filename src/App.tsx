import { useEffect, useState } from "react";
import { CustomTitlebar } from "@/components/titlebar";
import { MainInterface } from "@/components/interfaces/main";
import { CreditsInterface } from "@/components/interfaces/credits";
import { getAppSettings, setAppSettings } from "./lib/tauri";

type ViewMode = "main" | "credits";

export default function App() {
  const [viewMode, setViewMode] = useState<ViewMode>("main");
  const [selectedFolder, setSelectedFolder] = useState<string>("");
  const [appId, setAppId] = useState<string>("");
  const [theme, setTheme] = useState<"light" | "dark">("light");
  const [isThemeLoaded, setIsThemeLoaded] = useState<boolean>(false);

  useEffect(() => {
    document.documentElement.className =
      "min-h-screen bg-background flex flex-col custom-scrollbar fade-scrollbar";

    getAppSettings()
      .then((settings) => {
        setTheme(settings.theme);
        applyTheme(settings.theme);
        setIsThemeLoaded(true);
      })
      .catch(() => {
        console.error("Failed to load settings");
        setTheme("light");
        applyTheme("light");
        setIsThemeLoaded(true);
      });
  }, []);

  const applyTheme = (theme: "light" | "dark") => {
    document.documentElement.className = `min-h-screen bg-background flex flex-col custom-scrollbar fade-scrollbar ${
      theme === "dark" ? "dark" : ""
    }`;
  };

  const toggleTheme = async () => {
    const nextTheme = theme === "light" ? "dark" : "light";
    try {
      await setAppSettings({ theme: nextTheme });
      setTheme(nextTheme);
      applyTheme(nextTheme);
    } catch (error) {
      console.error("Failed to toggle theme:", error);
    }
  };

  // This is a quick fix to prevent flashing of unstyled content
  // while the theme is being loaded from the backend.
  if (!isThemeLoaded) {
    return null;
  }

  return (
    <div className={document.documentElement.className}>
      <CustomTitlebar
        onViewChange={setViewMode}
        currentView={viewMode}
        isDark={theme === "dark"}
        onToggleTheme={toggleTheme}
      />
      <div className="flex-1 pt-14 flex items-center justify-center p-8">
        {viewMode === "main" && (
          <MainInterface
            selectedFolder={selectedFolder}
            setSelectedFolder={setSelectedFolder}
            appId={appId}
            setAppId={setAppId}
          />
        )}
        {viewMode === "credits" && <CreditsInterface />}
      </div>
    </div>
  );
}
