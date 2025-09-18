import { getCurrentWindow } from "@tauri-apps/api/window";
import { getVersion } from "@tauri-apps/api/app";
import { Button } from "@/components/ui/button";
import { useEffect, useState } from "react";
import {
  X,
  Minimize,
  Users,
  Sun,
  Moon,
} from "lucide-react";

type ViewMode = "main" | "credits";

type CustomTitlebarProps = {
  onViewChange: (view: ViewMode) => void;
  currentView: ViewMode;
  onToggleTheme: () => void;
  isDark: boolean;
};

export function CustomTitlebar({
  onViewChange,
  currentView,
  onToggleTheme,
  isDark,
}: CustomTitlebarProps) {
  const appWindow = getCurrentWindow();
  const [version, setVersion] = useState("");

  useEffect(() => {
    getVersion()
      .then((v) => setVersion(v))
      .catch((err) => console.error("Failed to fetch version:", err));
  }, []);

  const handleMinimize = () => {
    appWindow.minimize();
  };

  const handleClose = () => {
    appWindow.close();
  };

  return (
    <div className="titlebar-drag fixed top-0 left-0 right-0 flex items-center justify-between h-14 bg-card/80 backdrop-blur-md border-b border-border/50 px-6 z-50">
      <div className="titlebar-no-drag flex items-center gap-2">
        <Button
          variant={currentView === "credits" ? "default" : "ghost"}
          size="sm"
          onClick={() =>
            onViewChange(currentView === "credits" ? "main" : "credits")
          }
          className={`h-8 w-8 p-0 hover:bg-muted/50 transition-colors duration-200 ${
            currentView === "credits"
              ? "hover:bg-primary/90 hover:text-primary-foreground"
              : "bg-transparent"
          }`}
          aria-label="Toggle View"
        >
          <Users
            className={`h-4 w-4 ${
              currentView === "credits"
                ? "text-primary-foreground"
                : "text-primary"
            }`}
          />
        </Button>
      </div>

      <div className="flex-1 flex justify-center">
        <button
          onClick={() => onViewChange("main")}
          className="titlebar-no-drag group relative"
          aria-label="Title"
        >
          <h1 className="text-xl font-bold text-foreground select-none tracking-tight transition-colors duration-200 px-4 py-1 rounded-md">
            Better Steam AutoCracker{" "}
            <span className="px-2 py-1 rounded-md bg-accent/60 text-sm font-normal text-muted-foreground">
              {version || ""}
            </span>
          </h1>
        </button>
      </div>

      <div className="titlebar-no-drag flex items-center gap-4">
        <Button
          variant="ghost"
          size="sm"
          onClick={onToggleTheme}
          aria-label="Toggle Dark Mode"
          className="h-8 w-8 p-0 rounded transition-colors duration-200"
        >
          {isDark ? (
            <Sun className="h-5 w-5 text-white" />
          ) : (
            <Moon className="h-5 w-5 text-black" />
          )}
        </Button>

        <Button
          variant="ghost"
          size="sm"
          className="h-8 w-8 p-0 hover:bg-muted/50 transition-colors duration-200"
          onClick={handleMinimize}
          aria-label="Minimize"
        >
          <Minimize className="h-4 w-4 text-gray-600 dark:text-gray-300" />
        </Button>

        <Button
          variant="ghost"
          size="sm"
          className="h-8 w-8 p-0 hover:bg-red-500 hover:text-white transition-colors duration-200"
          onClick={handleClose}
          aria-label="Close"
        >
          <X className="h-4 w-4 text-gray-600 dark:text-gray-300 group-hover:text-white" />
        </Button>
      </div>
    </div>
  );
}
