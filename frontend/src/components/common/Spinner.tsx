import clsx from "clsx";
import { Loader2 } from "lucide-react";

interface SpinnerProps {
  size?: "sm" | "md" | "lg";
  className?: string;
}

export function Spinner({ size = "md", className }: SpinnerProps) {
  return (
    <Loader2
      className={clsx(
        "animate-spin text-blue-500",
        {
          "w-4 h-4": size === "sm",
          "w-6 h-6": size === "md",
          "w-8 h-8": size === "lg",
        },
        className,
      )}
    />
  );
}

interface LoadingOverlayProps {
  message?: string;
}

export function LoadingOverlay({
  message = "Loading...",
}: LoadingOverlayProps) {
  return (
    <div className="flex flex-col items-center justify-center h-full w-full py-12">
      <Spinner size="lg" />
      <p className="mt-4 text-sm text-gray-400">{message}</p>
    </div>
  );
}
