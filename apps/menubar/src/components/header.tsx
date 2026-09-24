import { Plus, Settings } from "lucide-react";
import { Button } from "@/components/ui/button";

interface Props {
  onAdd: () => void;
  onSettings: () => void;
}

export function Header({ onAdd, onSettings }: Props) {
  return (
    <header className="flex h-11 items-center gap-2 pr-3 pl-[76px]">
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
