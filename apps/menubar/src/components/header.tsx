import { Plus, Settings } from "lucide-react";
import { Button } from "@/components/ui/button";

interface Props {
  onAdd: () => void;
  onSettings: () => void;
}

// macOS's overlay title bar draws the traffic lights on top of the
// webview, so the header needs left padding to clear them. Windows/Linux
// window chrome sits in its own space above the content instead — that
// padding there would just be a dead gap.
const isMac = typeof navigator !== "undefined" && navigator.platform.toLowerCase().includes("mac");

export function Header({ onAdd, onSettings }: Props) {
  return (
    <header className={`flex h-11 items-center gap-2 pr-3 ${isMac ? "pl-[76px]" : "pl-3"}`}>
      <span className="text-xs font-semibold text-muted-foreground">Mapper</span>
      <div className="ml-auto flex items-center gap-1">
        <Button variant="ghost" size="icon-sm" title="New tunnel" onClick={onAdd}>
          <Plus className="size-4" />
        </Button>
        <Button variant="ghost" size="icon-sm" title="Settings" onClick={onSettings}>
          <Settings className="size-4" />
        </Button>
      </div>
    </header>
  );
}
