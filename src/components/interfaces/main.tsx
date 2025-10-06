import { useState, useEffect, useRef } from "react";
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
  Hash,
} from "lucide-react";
import { selectDirectory, applyCrack, searchGame, Game } from "@/lib/tauri";
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

  const [searchTerm, setSearchTerm] = useState("");
  const [searchResults, setSearchResults] = useState<Game[]>([]);
  const [isSearching, setIsSearching] = useState(false);
  const [lastSearchTerm, setLastSearchTerm] = useState("");
  const isSearchInProgress = useRef(false);

  useEffect(() => {
    let unlisten: () => void;

    const setupListener = async () => {
      unlisten = await listen<{ progress: number; message: string }>(
        "crack-progress",
        (event) => {
          setProgress(event.payload.progress);
          setStatus(event.payload.message);
        },
      );
    };

    setupListener();

    return () => unlisten && unlisten();
  }, []);

  useEffect(() => {
    const handler = setTimeout(async () => {
      if (!searchTerm.trim()) {
        setSearchResults([]);
        setLastSearchTerm("");
        console.log(`[Search] Cleared search results for empty term.`);
        isSearchInProgress.current = false;
        return;
      }
      if (searchTerm !== lastSearchTerm && !isSearchInProgress.current) {
        isSearchInProgress.current = true;
        setIsSearching(true);
        console.log(`[Search] Searching for: ${searchTerm}`);
        try {
          const results = await searchGame(searchTerm);
          console.log(`[Search] Found ${results.length} relevant results.`);
          setSearchResults(results);
          setLastSearchTerm(searchTerm);
        } catch (error) {
          console.error(`[Search] Error searching for "${searchTerm}":`, error);
        } finally {
          setIsSearching(false);
          isSearchInProgress.current = false;
        }
      }
    }, 2000);

    return () => clearTimeout(handler);
  }, [searchTerm, lastSearchTerm]);

  useEffect(() => {
    const checkDRM = async () => {
      if (!appId || drmCheckAttempts >= 3 || isDrmChecking) {
        return;
      }
      setIsDrmChecking(true);
      setDrmCheckAttempts((prev) => prev + 1);
      try {
        const hasDenuvo = await checkForDRM(appId, drmCheckAttempts);
        setDrmWarning(
          hasDenuvo
            ? "This game contains Denuvo Anti-Tamper, you'll have to first manually crack the game's executable and afterwards use the program, otherwise the game won't work."
            : "",
        );
      } catch {
        setDrmWarning(
          `Failed to check DRM status. Attempt ${drmCheckAttempts} of 3.`,
        );
      } finally {
        setIsDrmChecking(false);
      }
    };

    checkDRM();
  }, [appId, drmCheckAttempts]);

  const handleFolderSelect = async () => {
    const folderPath = await selectDirectory();
    folderPath && setSelectedFolder(folderPath);
  };

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
      await applyCrack(appId, selectedFolder);
      setShowSuccessToast(true);
    } catch (error) {
      setStatus(
        `Error: ${error instanceof Error ? error.message : String(error)}`,
      );
    } finally {
      setIsProcessing(false);
      setIsCrackInitiated(false);
    }
  };

  const handleGameSelect = (game: Game) => {
    console.log(`[Search] Selected game: ${game.name} (AppID: ${game.appid})`);
    setAppId(game.appid.toString());
    setSearchTerm(game.name);
    setLastSearchTerm(game.name);
    setSearchResults([]);
  };

  const handleAppIdSearch = (appIdNum: number) => {
    console.log(`[Search] Using AppID: ${appIdNum}`);
    setAppId(appIdNum.toString());
    setSearchTerm(`AppID: ${appIdNum}`);
    setSearchResults([]);
  };

  // Checks if seach term is a number (AppID)
  const isAppIdInput = /^\d+$/.test(searchTerm.trim());
  const appIdNum = isAppIdInput ? parseInt(searchTerm.trim()) : null;

  return (
    <div className="flex-1 flex items-center justify-center min-h-0">
      <Card className="w-full max-w-3xl border-border bg-card shadow-2xl">
        <CardHeader className="text-center pb-8">
          <CardTitle className="text-3xl font-bold text-foreground">
            Get Started
          </CardTitle>
          <CardDescription className="text-lg text-muted-foreground/90">
            Configure the location and select your game.
          </CardDescription>
        </CardHeader>
        <CardContent className="space-y-8">
          <div className="space-y-4">
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
                className="flex-1 h-12"
              />
              <Button
                onClick={handleFolderSelect}
                className="flex items-center gap-2 h-12 px-6"
              >
                <FolderOpen className="h-5 w-5" />
                Browse
              </Button>
            </div>
          </div>

          <div className="space-y-4 relative">
            <Label
              htmlFor="search"
              className="text-base font-semibold text-foreground"
            >
              Search Steam Game
            </Label>
            <div className="flex gap-2 items-center">
              <Input
                id="search"
                placeholder="Search your game to crack... (e.g. 'Hollow Knight: Silksong')"
                value={searchTerm}
                onChange={(e) => setSearchTerm(e.target.value)}
                className="flex-1 h-12 border focus:ring-2 focus:ring-primary/50 transition-all duration-200"
              />
              {isSearching && <Loader2Icon className="h-5 w-5 animate-spin" />}
            </div>

            {/* AppID detection, still needs some work */}
            {isAppIdInput && appIdNum && (
              <div className="mt-2">
                <Button
                  onClick={() => handleAppIdSearch(appIdNum)}
                  variant="outline"
                  size="sm"
                  className="flex items-center gap-2"
                >
                  <Hash className="h-4 w-4" />
                  Use AppID: {appIdNum}
                </Button>
              </div>
            )}

            {searchResults.length > 0 && (
              <div className="absolute z-50 w-full mt-1 max-h-48 overflow-y-auto shadow-lg bg-card border border-border rounded-xl p-2 transition-all duration-300">
                {searchResults.map((game) => (
                  <div
                    key={game.appid}
                    className="w-full text-left px-4 py-3 bg-card hover:bg-primary/30 hover:text-primary-foreground cursor-pointer rounded-lg mb-2 last:mb-0 flex items-center justify-between transition-all duration-200"
                    onClick={() => handleGameSelect(game)}
                  >
                    <span className="font-medium text-lg truncate group-hover:text-primary-foreground transition-colors">
                      {game.name}
                    </span>
                    <span className="text-sm text-muted-foreground group-hover:text-muted-foreground/80 transition-colors">
                      {game.appid}
                    </span>
                  </div>
                ))}
              </div>
            )}
          </div>

          {drmWarning && (
            <div
              className={cn(
                "flex items-center gap-2 p-3 rounded-md text-sm",
                drmWarning.includes("No DRM")
                  ? "bg-primary/10 border-primary/20 text-primary"
                  : "bg-destructive/10 border-destructive/20 text-destructive",
              )}
            >
              {drmWarning.includes("No DRM") ? (
                <CheckCircle2Icon className="h-5 w-5 text-primary" />
              ) : (
                <AlertTriangleIcon className="h-5 w-5 text-destructive" />
              )}
              <span>{drmWarning}</span>
            </div>
          )}

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

          <Button
            onClick={handleStart}
            disabled={
              !appId ||
              !selectedFolder ||
              isCrackInitiated ||
              isProcessing ||
              isDrmChecking
            }
            className="w-full flex items-center justify-center gap-3 h-14 text-lg font-semibold bg-primary hover:bg-primary/90 disabled:opacity-50 transition-all duration-200 text-primary-foreground"
          >
            {(isProcessing || isDrmChecking) && (
              <Loader2Icon className="h-5 w-5 animate-spin text-primary-foreground" />
            )}
            Crack
          </Button>
        </CardContent>
      </Card>
      <SuccessToast
        visible={showSuccessToast}
        onClose={() => setShowSuccessToast(false)}
      />
    </div>
  );
}
