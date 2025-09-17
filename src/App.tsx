"use client";

import { useState } from "react";
import { CustomTitlebar } from "@/components/titlebar";
import { MainInterface } from "@/components/interfaces/main";
import { CreditsInterface } from "@/components/interfaces/credits";

type ViewMode = "main" | "credits";

export default function App() {
  const [viewMode, setViewMode] = useState<ViewMode>("main");
  const [selectedFolder, setSelectedFolder] = useState<string>("");
  const [appId, setAppId] = useState<string>("");

  return (
    <div className="min-h-screen bg-background flex flex-col custom-scrollbar fade-scrollbar">
      <CustomTitlebar onViewChange={setViewMode} currentView={viewMode} />

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
