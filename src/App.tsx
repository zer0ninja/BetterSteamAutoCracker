"use client";

import { useEffect, useState } from "react";
import { CustomTitlebar } from "@/components/titlebar";
import { MainInterface } from "@/components/interfaces/main";
import { CreditsInterface } from "@/components/interfaces/credits";
import { invoke } from "@tauri-apps/api/core";

type ViewMode = "main" | "credits";

export default function App() {
  const [viewMode, setViewMode] = useState<ViewMode>("main");
  const [selectedFolder, setSelectedFolder] = useState<string>("");
  const [appId, setAppId] = useState<string>("");

  const [isDark, setIsDark] = useState(false);

  useEffect(() => {
    invoke<string>("get_windows_theme")
      .then((theme) => {
        setIsDark(theme === "dark");
      })
      .catch(() => {
        setIsDark(false);
      });
  }, []);

  const toggleTheme = () => {
    setIsDark((prev) => !prev);
  };

  return (
    <div className={`min-h-screen bg-background flex flex-col custom-scrollbar fade-scrollbar ${isDark ? "dark" : ""}`}>
      <CustomTitlebar
        onViewChange={setViewMode}
        currentView={viewMode}
        isDark={isDark}
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