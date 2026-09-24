import { useState } from "react";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { useTunnelLog } from "../hooks/useTunnelLog";
import type { Tunnel } from "../types";

interface Props {
  tunnels: Tunnel[];
  onStart: (id: string) => void;
  onStop: (id: string) => void;
  onDelete: (id: string) => void;
  onExport: () => void;
  onImport: () => void;
}

export function TunnelList({ tunnels, onStart, onStop, onDelete, onExport, onImport }: Props) {
  const [expandedId, setExpandedId] = useState<string | null>(null);

  return (
    <section className="flex min-h-0 flex-1 flex-col gap-2.5 overflow-y-auto p-4">
      <div className="flex items-center gap-2">
        <h2 className="text-xs font-semibold tracking-wide text-muted-foreground uppercase">
          Tunnels
        </h2>
        <Badge variant="secondary">{tunnels.length}</Badge>
        <div className="ml-auto flex gap-1.5">
          <Button size="sm" variant="ghost" onClick={onImport}>
            Import
          </Button>
          <Button size="sm" variant="ghost" onClick={onExport} disabled={tunnels.length === 0}>
            Export
          </Button>
        </div>
      </div>

      {tunnels.length === 0 && (
        <p className="text-xs text-muted-foreground">
          No tunnels yet. Start one below — it's remembered here.
        </p>
      )}

      <ul className="flex flex-col gap-2">
        {tunnels.map((t) => (
          <TunnelRow
            key={t.id}
            tunnel={t}
            expanded={expandedId === t.id}
            onToggleLog={() => setExpandedId(expandedId === t.id ? null : t.id)}
            onStart={() => onStart(t.id)}
            onStop={() => onStop(t.id)}
            onDelete={() => onDelete(t.id)}
          />
        ))}
      </ul>
    </section>
  );
}

function TunnelRow({
  tunnel: t,
  expanded,
  onToggleLog,
  onStart,
  onStop,
  onDelete,
}: {
  tunnel: Tunnel;
  expanded: boolean;
  onToggleLog: () => void;
  onStart: () => void;
  onStop: () => void;
  onDelete: () => void;
}) {
  const log = useTunnelLog(t.id, expanded && t.running);

  return (
    <li className="flex flex-col gap-2 rounded-lg border border-border bg-card px-2.5 py-2">
      <div className="flex items-center justify-between gap-2">
        <div className="flex min-w-0 flex-col gap-0.5">
          <div className="flex items-center gap-1.5">
            <span
              className={`size-1.5 shrink-0 rounded-full ${
                !t.running
                  ? "bg-muted-foreground/40"
                  : t.status === "connected"
                    ? "bg-emerald-500"
                    : "animate-pulse bg-amber-500"
              }`}
              title={!t.running ? "Stopped" : t.status === "connected" ? "Connected" : "Connecting…"}
            />
            <strong className="truncate text-xs font-semibold">
              {t.localPort} → {t.remoteHost}:{t.remotePort}
            </strong>
          </div>
          <span className="text-[10px] text-muted-foreground">
            via {t.sshHost}
            {t.running && t.latencyMs != null && ` · ${t.latencyMs}ms`}
          </span>
        </div>
        <div className="flex shrink-0 gap-1.5">
          {t.running && (
            <Button size="sm" variant="ghost" onClick={onToggleLog}>
              {expanded ? "Hide log" : "Log"}
            </Button>
          )}
          {t.running ? (
            <Button size="sm" variant="destructive" onClick={onStop}>
              Stop
            </Button>
          ) : (
            <>
              <Button size="sm" variant="outline" onClick={onStart}>
                Start
              </Button>
              <Button size="sm" variant="ghost" onClick={onDelete}>
                Delete
              </Button>
            </>
          )}
        </div>
      </div>

      {expanded && t.running && (
        <pre className="max-h-32 overflow-y-auto rounded-md bg-muted p-2 text-[10px] whitespace-pre-wrap text-muted-foreground">
          {log.length > 0 ? log.join("\n") : "No output yet."}
        </pre>
      )}
    </li>
  );
}
