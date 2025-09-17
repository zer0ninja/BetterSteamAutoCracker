import { CheckCircle2Icon } from "lucide-react";
import { AlertDescription, AlertTitle } from "@/components/ui/alert";
import { useEffect, useState } from "react";
import { cn } from "@/lib/utils";

interface SuccessToastProps {
  message?: string;
  visible: boolean;
  duration?: number;
  onClose?: () => void;
}

export const SuccessToast = ({
  message = "Emulator has been applied.",
  visible,
  duration = 4000,
  onClose,
}: SuccessToastProps) => {
  const [show, setShow] = useState(visible);

  useEffect(() => {
    if (visible) {
      setShow(true);
      const timeout = setTimeout(() => {
        setShow(false);
        onClose?.();
      }, duration);
      return () => clearTimeout(timeout);
    }
  }, [visible, duration, onClose]);

  return (
    <div
      className={cn(
        "fixed bottom-6 right-6 z-[9999] transform-gpu transition-all duration-500 ease-in-out",
        show
          ? "opacity-100 translate-y-0"
          : "opacity-0 translate-y-4 pointer-events-none"
      )}
    >
      <div className="flex items-start gap-4 p-4 pr-6 rounded-lg bg-green-50 border border-green-200 shadow-xl w-[360px]">
        <div className="mt-1 text-green-600">
          <CheckCircle2Icon className="h-6 w-6" />
        </div>
        <div className="flex-1">
          <AlertTitle className="text-base font-semibold text-green-800">
            Success!
          </AlertTitle>
          <AlertDescription className="mt-1 text-sm text-green-700 leading-snug">
            {message}
          </AlertDescription>
        </div>
      </div>
    </div>
  );
};
