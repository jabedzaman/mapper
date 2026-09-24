import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import type { TunnelInfo } from "../types";

interface Props {
  tunnels: TunnelInfo[];
  onStop: (id: string) => void;
}

export function TunnelList({ tunnels, onStop }: Props) {
  return (
    <section className="flex min-h-0 flex-1 flex-col gap-2.5 overflow-y-auto p-4">
      <div className="flex items-center gap-2">
        <h2 className="text-xs font-semibold tracking-wide text-muted-foreground uppercase">
          Active
        </h2>
        <Badge variant="secondary">{tunnels.length}</Badge>
      </div>

      {tunnels.length === 0 && (
        <p className="text-xs text-muted-foreground">No tunnels running.</p>
      )}

      <ul className="flex flex-col gap-2">
        {tunnels.map((t) => (
          <li
            key={t.id}
            className="flex items-center justify-between gap-2 rounded-lg border border-border bg-card px-2.5 py-2"
          >
            <div className="flex min-w-0 flex-col gap-0.5">
              <strong className="text-xs font-semibold">
                {t.localPort} → {t.remoteHost}:{t.remotePort}
              </strong>
              <span className="text-[10px] text-muted-foreground">via {t.sshHost}</span>
            </div>
            <Button
              size="sm"
              variant="destructive"
              className="shrink-0"
              onClick={() => onStop(t.id)}
            >
              Stop
            </Button>
          </li>
        ))}
      </ul>
    </section>
  );
}
