import { Button } from "@/components/ui/button";
import { api } from "../lib/api";

export function Header() {
  return (
    <header
      data-tauri-drag-region
      className="flex h-11 items-center gap-2 border-b border-border pr-4 pl-[76px]"
    >
      <span data-tauri-drag-region className="text-xs text-muted-foreground">
        SSH port forwarding
      </span>
      <Button
        variant="outline"
        size="sm"
        className="ml-auto"
        onClick={() => api.quitApp()}
      >
        Quit
      </Button>
    </header>
  );
}
