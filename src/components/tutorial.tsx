"use client";

import { useState } from "react";
import { Button } from "@/components/ui/button";
import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from "@/components/ui/card";
import { ChevronRight, ChevronLeft } from "lucide-react";

interface TutorialOverlayProps {
  onClose: () => void;
}

export function TutorialOverlay({ onClose }: TutorialOverlayProps) {
  const [currentStep, setCurrentStep] = useState(0);

  const steps = [
    {
      title: 'Say, "hi" to Better Steam AutoCracker!',
      description: "Your tool for auto-cracking Steam games.",
      content:
        "Steam AutoCracker helps you to crack Steam games. This guide will tell you how to pick a game folder, enter an App ID, and crack the game.",
    },
    {
      title: "Choose Game Folder",
      description: "Select where your game is stored.",
      content:
        "Click 'Browse' next to the Game Folder field. Pick the folder with your game files, usually in 'Steam/steamapps/common'. This tells the app where to find your game.",
    },
    {
      title: "Enter App ID",
      description: "Type in the game's Steam App ID.",
      content:
        "Find the game's App ID on its Steam store page (like '1030300' in 'store.steampowered.com/app/1030300/') or on SteamDB. Enter it in the input field to link the game.",
    },
    {
      title: "Start Cracking",
      description: "Click to begin the cracking process.",
      content:
        "After picking the folder and App ID, hit 'Crack'. A progress bar will show how it's going. The app tweaks the game to run without Steam. Wait for it to finish and check for any errors.",
    },
    {
      title: "See Credits",
      description: "Check out who built this tool!",
      content:
        "Click the users icon in the titlebar to view the credits. You'll see who made Better Steam AutoCracker and what they've used.",
    },
  ];

  const nextStep = () => {
    if (currentStep < steps.length - 1) {
      setCurrentStep(currentStep + 1);
    } else {
      onClose();
    }
  };

  const prevStep = () => {
    if (currentStep > 0) {
      setCurrentStep(currentStep - 1);
    }
  };

  return (
    <div className="fixed inset-0 bg-background/80 backdrop-blur-sm z-50 flex items-center justify-center p-4 md:p-6">
      <Card className="w-full max-w-md border-border bg-card shadow-lg rounded-lg overflow-hidden transition-all duration-300">
        <CardHeader className="flex flex-row items-center justify-between space-y-0 p-6 border-b border-border/50">
          <div>
            <CardTitle className="text-xl font-semibold text-foreground">
              {steps[currentStep].title}
            </CardTitle>
            <CardDescription className="text-sm text-muted-foreground mt-4">
              Step {currentStep + 1} of {steps.length}
            </CardDescription>
          </div>
        </CardHeader>
        <CardContent className="p-6 space-y-6">
          <div className="animate-fade-in">
            <p className="text-muted-foreground text-base font-medium mb-4">
              {steps[currentStep].description}
            </p>
            <p className="text-sm text-muted-foreground leading-relaxed">
              {steps[currentStep].content}
            </p>
          </div>

          <div className="space-y-6">
            <div className="w-full h-1 bg-muted rounded-full overflow-hidden">
              <div
                className="h-full bg-red-600 transition-all duration-300 ease-in-out"
                style={{
                  width: `${((currentStep + 1) / steps.length) * 100}%`,
                }}
              />
            </div>
            <div className="flex items-center justify-between">
              <Button
                variant="outline"
                onClick={prevStep}
                disabled={currentStep === 0}
                className="flex items-center gap-2 bg-transparent border-border text-foreground hover:bg-muted hover:border-border/80 transition-colors duration-200"
              >
                <ChevronLeft className="h-4 w-4" />
                Previous
              </Button>

              <div className="flex gap-3">
                {steps.map((_, index) => (
                  <div
                    key={index}
                    className={`w-2 h-2 rounded-full transition-colors ${
                      index === currentStep ? "bg-red-600" : "bg-muted"
                    }`}
                  />
                ))}
              </div>

              <Button
                onClick={nextStep}
                className="flex items-center gap-2 bg-red-600 hover:bg-red-700 text-white font-medium transition-colors duration-200"
              >
                {currentStep === steps.length - 1 ? "Get Started" : "Next"}
                {currentStep < steps.length - 1 && (
                  <ChevronRight className="h-4 w-4" />
                )}
              </Button>
            </div>
          </div>
        </CardContent>
      </Card>
    </div>
  );
}
