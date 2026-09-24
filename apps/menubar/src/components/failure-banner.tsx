import { useState } from "react";
import { Alert, AlertDescription } from "@/components/ui/alert";
import { Button } from "@/components/ui/button";
import { api } from "../lib/api";
import type { OrphanedProcess, TunnelFailure } from "../types";

interface Props {
  failure: TunnelFailure;
  freeing: boolean;
  onFreeAndRetry: (failure: TunnelFailure) => void;
}

export function FailureBanner({ failure, freeing, onFreeAndRetry }: Props) {
  const [owners, setOwners] = useState<OrphanedProcess[] | null>(null);
  const [loadingOwners, setLoadingOwners] = useState(false);

  async function startConfirm() {
    setLoadingOwners(true);
    try {
      setOwners(await api.getPortOwners(failure.localPort));
    } catch {
      setOwners([]);
    } finally {
      setLoadingOwners(false);
    }
  }

  function confirmed() {
    setOwners(null);
    onFreeAndRetry(failure);
  }

  return (
    <Alert variant="destructive" className="gap-2">
      <AlertDescription className="break-words text-destructive">
        {failure.sshHost}:{failure.localPort} → {failure.message}
      </AlertDescription>

      {failure.portInUse && owners === null && (
        <Button
          type="button"
          size="sm"
          variant="outline"
          disabled={freeing || loadingOwners}
          className="w-fit border-amber-500/40 bg-amber-500/10 text-amber-600 hover:bg-amber-500/20 dark:text-amber-400"
          onClick={startConfirm}
        >
          {loadingOwners ? "Checking port…" : `Kill process on port ${failure.localPort} & retry`}
        </Button>
      )}

      {owners !== null && (
        <div className="flex flex-col gap-2">
          {owners.length > 0 ? (
            <ul className="flex flex-col gap-0.5 text-[10px] text-muted-foreground">
              {owners.map((o) => (
                <li key={o.pid} className="truncate">
                  pid {o.pid} · {o.command || "unknown process"}
                </li>
              ))}
            </ul>
          ) : (
            <p className="text-[10px] text-muted-foreground">
              Couldn't determine what's using the port — it may have already exited.
            </p>
          )}
          <div className="flex gap-2">
            <Button
              type="button"
              size="sm"
              variant="destructive"
              disabled={freeing}
              onClick={confirmed}
            >
              {freeing ? "Freeing port…" : "Kill & retry"}
            </Button>
            <Button type="button" size="sm" variant="ghost" disabled={freeing} onClick={() => setOwners(null)}>
              Cancel
            </Button>
          </div>
        </div>
      )}
    </Alert>
  );
}
