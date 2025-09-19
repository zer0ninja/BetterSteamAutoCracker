import { useState, useEffect } from "react";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Progress } from "@/components/ui/progress";
import {
  Card,
  CardContent,
  CardHeader,
  CardTitle,
  CardDescription,
} from "@/components/ui/card";
import { Label } from "@/components/ui/label";
import {
  FolderOpen,
  CheckCircle2Icon,
  AlertTriangleIcon,
  Loader2Icon,
} from "lucide-react";
import { selectDirectory, applyCrack } from "@/lib/tauri";
import { checkForDRM } from "@/lib/checkForDRM";
import { listen } from "@tauri-apps/api/event";
import { SuccessToast } from "@/components/toast";
import { cn } from "@/lib/utils";

interface MainInterfaceProps {
  selectedFolder: string;
  setSelectedFolder: (folder: string) => void;
  appId: string;
  setAppId: (appId: string) => void;
}

export function MainInterface({
  selectedFolder,
  setSelectedFolder,
  appId,
  setAppId,
}: MainInterfaceProps) {
  const [isProcessing, setIsProcessing] = useState(false);
  const [isCrackInitiated, setIsCrackInitiated] = useState(false);
  const [progress, setProgress] = useState(0);
  const [status, setStatus] = useState<string>("");
  const [drmWarning, setDrmWarning] = useState<string>("");
  const [showSuccessToast, setShowSuccessToast] = useState(false);
  const [isDrmChecking, setIsDrmChecking] = useState(false);
  const [drmCheckAttempts, setDrmCheckAttempts] = useState(0);

  const handleFolderSelect = async () => {
    try {
      const folderPath = await selectDirectory();
      if (folderPath) {
        setSelectedFolder(folderPath);
        console.log(
          `[${new Date().toISOString()}] Selected folder: ${folderPath}`
        );
      }
    } catch (error) {
      console.error(
        `[${new Date().toISOString()}] Failed to select folder:`,
        error
      );
    }
  };

  useEffect(() => {
    let unlisten: () => void;

    const setupListener = async () => {
      unlisten = await listen<{ progress: number; message: string }>(
        "crack-progress",
        (event) => {
          console.log(
            `[${new Date().toISOString()}] Received crack-progress:`,
            event.payload
          );
          setProgress(event.payload.progress);
          setStatus(event.payload.message);
        }
      );
    };

    setupListener().catch((error) =>
      console.error(
        `[${new Date().toISOString()}] Failed to setup listener:`,
        error
      )
    );

    return () => {
      if (unlisten) {
        unlisten();
        console.log(`[${new Date().toISOString()}] Event listener cleaned up`);
      }
    };
  }, []);

  useEffect(() => {
    const checkDRM = async () => {
      if (!appId || drmCheckAttempts >= 3) {
        setIsDrmChecking(false);
        return;
      }

      setIsDrmChecking(true);
      setDrmCheckAttempts((prev) => prev + 1);

      try {
        const hasDenuvo = await checkForDRM(appId, drmCheckAttempts);
        if (hasDenuvo) {
          setDrmWarning(
            "This game contains Denuvo Anti-Tamper, you'll have to first manually crack the game's executable and afterwards use the program, otherwise the game won't work."
          );
        } else {
          setDrmWarning("No DRM detected.");
        }
      } catch (error) {
        console.error(
          `[${new Date().toISOString()}] Failed to check DRM for App ID ${appId}:`,
          error
        );
        setDrmWarning(
          `Failed to check DRM status. Attempt ${drmCheckAttempts} of 3.`
        );
      } finally {
        setIsDrmChecking(false);
      }
    };

    checkDRM();
  }, [appId, drmCheckAttempts]);

  const handleStart = async () => {
    if (
      !appId ||
      !selectedFolder ||
      isCrackInitiated ||
      isProcessing ||
      isDrmChecking
    )
      return;
    setIsCrackInitiated(true);
    setIsProcessing(true);
    setProgress(0);
    setStatus("Started");

    try {
      const result = await applyCrack(appId, selectedFolder);
      console.log(`[${new Date().toISOString()}] Cracking completed:`, result);
      setShowSuccessToast(true);
    } catch (error) {
      console.error(`[${new Date().toISOString()}] Cracking failed:`, error);
      setStatus(
        `Error: ${error instanceof Error ? error.message : String(error)}`
      );
    } finally {
      console.log(`[${new Date().toISOString()}] Crack process finished`);
      setIsProcessing(false);
      setIsCrackInitiated(false);
    }
  };

  return (
    <div className="flex-1 flex items-center justify-center min-h-0">
      <Card className="w-full max-w-2xl border-border bg-card shadow-2xl">
        <CardHeader className="text-center pb-8">
          <CardTitle className="text-3xl font-bold text-foreground">
            Get Started
          </CardTitle>
          <CardDescription className="text-lg text-muted-foreground/90">
            Configure the location and App ID.
          </CardDescription>
        </CardHeader>
        <CardContent className="space-y-8">
          <div className="space-y-4" id="game-folder-section">
            <Label
              htmlFor="folder"
              className="text-base font-semibold text-foreground"
            >
              Game Folder
            </Label>
            <div className="flex gap-4">
              <Input
                id="folder"
                placeholder="Select your game folder..."
                value={selectedFolder}
                readOnly
                className="flex-1 h-12 bg-background border-border text-base text-foreground placeholder:text-muted-foreground/70 focus:border-primary focus:ring-primary/20"
              />
              <Button
                onClick={handleFolderSelect}
                variant="outline"
                className="cursor-pointer flex items-center gap-2 h-12 px-6 bg-background border-border hover:bg-accent hover:border-primary transition-all duration-200 text-foreground"
              >
                <FolderOpen className="h-5 w-5" />
                Browse
              </Button>
            </div>
          </div>

          <div className="space-y-4" id="app-id-section">
            <Label
              htmlFor="appid"
              className="text-base font-semibold text-foreground"
            >
              Steam App ID
            </Label>
            <Input
              id="appid"
              placeholder="Enter Steam App ID (e.g., 1030300 for Hollow Knight: Silksong)"
              value={appId}
              onChange={(e) => setAppId(e.target.value)}
              className="h-12 bg-background border-border text-base text-foreground placeholder:text-muted-foreground/70 focus:border-primary focus:ring-primary/20"
            />
            {drmWarning && (
              <div
                className={cn(
                  "flex items-center gap-2 p-3 rounded-md text-sm",
                  drmWarning.includes("No DRM")
                    ? "bg-primary/10 border-primary/20 text-primary"
                    : "bg-destructive/10 border-destructive/20 text-destructive"
                )}
              >
                {drmWarning.includes("No DRM") ? (
                  <CheckCircle2Icon className={cn("h-5 w-5", "text-primary")} />
                ) : (
                  <AlertTriangleIcon
                    className={cn("h-5 w-5", "text-destructive")}
                  />
                )}
                <span>{drmWarning}</span>
              </div>
            )}
          </div>

          {isProcessing && (
            <div className="space-y-4">
              <div className="flex items-center justify-between">
                <Label className="text-base font-semibold text-foreground">
                  Progress
                </Label>
                <span className="text-sm font-medium text-primary">
                  {progress}%
                </span>
              </div>
              <Progress value={progress} className="w-full h-3" />
              <div className="text-sm text-muted-foreground">{status}</div>
            </div>
          )}

          {isProcessing ? null : (
            <Button
              onClick={handleStart}
              disabled={
                !appId ||
                !selectedFolder ||
                isProcessing ||
                isCrackInitiated ||
                isDrmChecking
              }
              className="w-full flex items-center justify-center gap-3 h-14 text-lg font-semibold bg-primary hover:bg-primary/90 disabled:opacity-50 transition-all duration-200 text-primary-foreground"
              size="lg"
              id="crack-button"
            >
              {isDrmChecking && (
                <Loader2Icon className="h-5 w-5 animate-spin text-primary-foreground" />
              )}
              Crack
            </Button>
          )}
        </CardContent>
      </Card>
      <SuccessToast
        visible={showSuccessToast}
        onClose={() => setShowSuccessToast(false)}
      />
    </div>
  );
}
