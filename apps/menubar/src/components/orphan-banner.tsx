import { Alert, AlertDescription } from "@/components/ui/alert";
import { Button } from "@/components/ui/button";
import type { OrphanedProcess } from "../types";

interface Props {
  orphans: OrphanedProcess[];
  killing: boolean;
  onKillAll: () => void;
}

export function OrphanBanner({ orphans, killing, onKillAll }: Props) {
  if (orphans.length === 0) return null;

  return (
    <Alert className="m-4 mb-0 w-auto gap-2 border-amber-500/40 bg-amber-500/10">
      <AlertDescription className="text-amber-700 dark:text-amber-400">
        {orphans.length} leftover ssh {orphans.length === 1 ? "process" : "processes"} from a
        previous run {orphans.length === 1 ? "is" : "are"} still running.
      </AlertDescription>
      <Button
        type="button"
        size="sm"
        variant="outline"
        disabled={killing}
        className="w-fit border-amber-500/40 bg-amber-500/10 text-amber-600 hover:bg-amber-500/20 dark:text-amber-400"
        onClick={onKillAll}
      >
        {killing ? "Killing…" : `Kill ${orphans.length} leftover process${orphans.length === 1 ? "" : "es"}`}
      </Button>
    </Alert>
  );
}
