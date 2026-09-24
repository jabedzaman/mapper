import { useEffect, useState } from "react";
import { Accordion as AccordionPrimitive } from "@base-ui/react/accordion";
import { Play, Square, ChevronDown, ChevronUp, Trash2, ExternalLink, Activity, ScrollText, Info, Pencil } from "lucide-react";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Accordion, AccordionItem, AccordionContent } from "@/components/ui/accordion";
import { Tabs, TabsList, TabsTrigger, TabsContent } from "@/components/ui/tabs";
import { useTunnelLog } from "../hooks/use-tunnel-log";
import { LatencyChart } from "./latency-chart";
import { api } from "../lib/api";
import type { Tunnel } from "../types";

const LATENCY_HISTORY_LIMIT = 60; // ~60s of samples at the 1s poll interval

interface Props {
  tunnels: Tunnel[];
  onStart: (id: string) => void;
  onStop: (id: string) => void;
  onDelete: (id: string) => void;
  onEdit: (id: string) => void;
}

export function TunnelList({ tunnels, onStart, onStop, onDelete, onEdit }: Props) {
  const [openIds, setOpenIds] = useState<string[]>([]);

  return (
    <section className="flex min-h-0 flex-1 flex-col gap-2.5 overflow-y-auto p-4">
      <div className="flex items-center gap-2">
        <h2 className="text-xs font-semibold tracking-wide text-muted-foreground uppercase">
          Tunnels
        </h2>
        <Badge variant="secondary">{tunnels.length}</Badge>
      </div>

      {tunnels.length === 0 && (
        <p className="text-xs text-muted-foreground">
          No tunnels yet. Start one below — it's remembered here.
        </p>
      )}

      {/* Single-open: no `multiple` prop, so opening a row closes whichever was open. */}
      <Accordion value={openIds} onValueChange={(v) => setOpenIds(v as string[])} className="gap-2">
        {tunnels.map((t) => (
          <TunnelRow
            key={t.id}
            tunnel={t}
            open={openIds.includes(t.id)}
            onStart={() => onStart(t.id)}
            onStop={() => onStop(t.id)}
            onDelete={() => onDelete(t.id)}
            onEdit={() => onEdit(t.id)}
          />
        ))}
      </Accordion>
    </section>
  );
}

function statusDotClass(t: Tunnel): string {
  if (!t.running) return "bg-muted-foreground/40";
  if (t.status === "connected") return "bg-emerald-500";
  if (t.status === "retrying") return "animate-pulse bg-red-500";
  return "animate-pulse bg-amber-500";
}

function fmtDuration(secs: number): string {
  if (secs < 60) return `${secs}s`;
  const m = Math.floor(secs / 60);
  if (m < 60) return `${m}m ${secs % 60}s`;
  const h = Math.floor(m / 60);
  if (h < 24) return `${h}h ${m % 60}m`;
  return `${Math.floor(h / 24)}d ${h % 24}h`;
}

function DetailRow({ label, value }: { label: string; value: string }) {
  return (
    <div className="flex items-baseline justify-between gap-3">
      <dt className="shrink-0 text-muted-foreground">{label}</dt>
      <dd className="truncate font-medium">{value}</dd>
    </div>
  );
}

function statusTitle(t: Tunnel): string {
  if (!t.running) return "Stopped";
  if (t.status === "connected") return "Connected";
  if (t.status === "retrying") return "Retrying…";
  return "Connecting…";
}

function TunnelRow({
  tunnel: t,
  open,
  onStart,
  onStop,
  onDelete,
  onEdit,
}: {
  tunnel: Tunnel;
  open: boolean;
  onStart: () => void;
  onStop: () => void;
  onDelete: () => void;
  onEdit: () => void;
}) {
  const log = useTunnelLog(t.id, open && t.running);
  const [latencyHistory, setLatencyHistory] = useState<number[]>([]);

  useEffect(() => {
    if (!t.running) {
      setLatencyHistory([]);
      return;
    }
    if (t.status === "connected" && t.latencyMs != null) {
      const ms = t.latencyMs;
      setLatencyHistory((prev) => [...prev.slice(-(LATENCY_HISTORY_LIMIT - 1)), ms]);
    }
  }, [t.running, t.status, t.latencyMs]);

  return (
    <AccordionItem value={t.id} className="rounded-lg border border-border bg-card px-2.5 not-last:border-b-0">
      <AccordionPrimitive.Header className="flex items-center gap-1">
        <AccordionPrimitive.Trigger className="group/trigger flex flex-1 items-center justify-between gap-2 py-2 text-left outline-none">
          <div className="flex min-w-0 flex-col gap-0.5">
            <div className="flex items-center gap-1.5">
              <span className={`size-1.5 shrink-0 rounded-full ${statusDotClass(t)}`} title={statusTitle(t)} />
              <strong className="truncate text-xs font-semibold">
                {t.localPort} → {t.remoteHost}:{t.remotePort}
              </strong>
            </div>
            <span className="text-[10px] text-muted-foreground">
              via {t.sshHost}
              {t.status === "retrying"
                ? ` · retrying in ${t.retryInSecs}s (attempt ${t.retryAttempt}/6)`
                : t.running && t.latencyMs != null && ` · ${t.latencyMs.toFixed(2)}ms`}
              {t.connectedSecs != null && ` · connected ${fmtDuration(t.connectedSecs)}`}
            </span>
          </div>
          <ChevronDown className="size-4 shrink-0 text-muted-foreground group-aria-expanded/trigger:hidden" />
          <ChevronUp className="hidden size-4 shrink-0 text-muted-foreground group-aria-expanded/trigger:inline" />
        </AccordionPrimitive.Trigger>

<div className="flex shrink-0 items-center gap-1">
            {t.running ? (
              <>
                <Button
                  size="icon-sm"
                  variant="outline"
                  title="Open in browser"
                  onClick={() => api.openInBrowser(t.localPort)}
                >
                  <ExternalLink className="size-4" />
                </Button>
                <Button size="icon-sm" variant="outline" title="Edit" onClick={onEdit}>
                  <Pencil className="size-4" />
                </Button>
                <Button size="icon-sm" variant="destructive" title="Stop" onClick={onStop}>
                  <Square className="size-4" />
                </Button>
              </>
            ) : (
            <>
              <Button size="icon-sm" variant="outline" title="Start" onClick={onStart}>
                <Play className="size-4" />
              </Button>
              <Button size="icon-sm" variant="outline" title="Edit" onClick={onEdit}>
                <Pencil className="size-4" />
              </Button>
              <Button size="icon-sm" variant="ghost" title="Delete" onClick={onDelete}>
                <Trash2 className="size-4" />
              </Button>
            </>
          )}
        </div>
      </AccordionPrimitive.Header>

      <AccordionContent>
        {t.running ? (
          <Tabs defaultValue="latency">
            <TabsList className="h-7 w-full">
              <TabsTrigger value="latency" className="gap-1 text-[11px]">
                <Activity className="size-3.5" />
                Latency
              </TabsTrigger>
              <TabsTrigger value="details" className="gap-1 text-[11px]">
                <Info className="size-3.5" />
                Details
              </TabsTrigger>
              <TabsTrigger value="log" className="gap-1 text-[11px]">
                <ScrollText className="size-3.5" />
                Log
              </TabsTrigger>
            </TabsList>
            <TabsContent value="latency" className="mt-2">
              <LatencyChart data={latencyHistory} />
            </TabsContent>
            <TabsContent value="details" className="mt-2">
              <dl className="flex flex-col gap-1 rounded-md bg-muted p-2 text-[10px]">
                <DetailRow label="SSH host" value={t.sshHost} />
                <DetailRow label="Forward" value={`localhost:${t.localPort} → ${t.remoteHost}:${t.remotePort}`} />
                <DetailRow label="Status" value={statusTitle(t)} />
                <DetailRow
                  label="Connected"
                  value={t.connectedSecs != null ? fmtDuration(t.connectedSecs) : "—"}
                />
                <DetailRow
                  label="Latency"
                  value={t.latencyMs != null ? `${t.latencyMs.toFixed(2)}ms` : "—"}
                />
                <DetailRow label="PID" value={t.pid != null ? String(t.pid) : "—"} />
              </dl>
            </TabsContent>
            <TabsContent value="log" className="mt-2">
              <pre className="max-h-32 overflow-y-auto rounded-md bg-muted p-2 text-[10px] whitespace-pre-wrap text-muted-foreground">
                {log.length > 0 ? log.join("\n") : "No output yet."}
              </pre>
            </TabsContent>
          </Tabs>
        ) : (
          <p className="text-[10px] text-muted-foreground">Stopped. Start it to see live status.</p>
        )}
      </AccordionContent>
    </AccordionItem>
  );
}
