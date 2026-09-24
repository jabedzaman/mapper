import { Plus, Power } from "lucide-react";
import { Button } from "@/components/ui/button";
import { api } from "../lib/api";

interface Props {
  onAdd: () => void;
}

export function Header({ onAdd }: Props) {
  return (
    <header className="flex h-11 items-center gap-2 pr-3 pl-[76px]">
      <span className="text-xs font-semibold text-muted-foreground">Mapper</span>
      <div className="ml-auto flex items-center gap-1">
        <Button variant="ghost" size="icon-sm" title="New tunnel" onClick={onAdd}>
          <Plus className="size-4" />
        </Button>
        <Button variant="ghost" size="icon-sm" title="Quit" onClick={() => api.quitApp()}>
          <Power className="size-4" />
        </Button>
      </div>
    </header>
  );
}
