import { useEffect, useState } from "react";
import { Accordion as AccordionPrimitive } from "@base-ui/react/accordion";
import {
  Play,
  Square,
  ChevronDown,
  ChevronUp,
  Trash2,
  Activity,
  ScrollText,
  Info,
  Pencil,
  Gauge,
  Clock,
  ArrowDown,
  ArrowUp,
  Server,
  Hash,
  ArrowLeftRight,
  type LucideIcon,
} from "lucide-react";
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

function statusPillClass(t: Tunnel): string {
  if (!t.running) return "bg-muted text-muted-foreground";
  if (t.status === "connected") return "bg-emerald-500/15 text-emerald-600 dark:text-emerald-400";
  if (t.status === "retrying") return "bg-red-500/15 text-red-600 dark:text-red-400";
  return "bg-amber-500/15 text-amber-600 dark:text-amber-400";
}

function statusDotClass(t: Tunnel): string {
  if (!t.running) return "bg-muted-foreground/40";
  if (t.status === "connected") return "bg-emerald-500";
  if (t.status === "retrying") return "animate-pulse bg-red-500";
  return "animate-pulse bg-amber-500";
}

function fmtBytes(bytes: number): string {
  if (bytes < 1024) return `${bytes}B`;
  const units = ["KB", "MB", "GB", "TB"];
  let value = bytes / 1024;
  let unit = 0;
  while (value >= 1024 && unit < units.length - 1) {
    value /= 1024;
    unit += 1;
  }
  return `${value.toFixed(value < 10 ? 1 : 0)}${units[unit]}`;
}

function fmtDuration(secs: number): string {
  if (secs < 60) return `${secs}s`;
  const m = Math.floor(secs / 60);
  if (m < 60) return `${m}m ${secs % 60}s`;
  const h = Math.floor(m / 60);
  if (h < 24) return `${h}h ${m % 60}m`;
  return `${Math.floor(h / 24)}d ${h % 24}h`;
}

function DetailCell({ icon: Icon, label, value }: { icon: LucideIcon; label: string; value: string }) {
  return (
    <div className="flex min-w-0 items-center gap-2 rounded-md border border-border/60 bg-muted/40 p-2">
      <Icon className="size-3.5 shrink-0 text-muted-foreground" />
      <div className="flex min-w-0 flex-col">
        <dt className="text-[9px] text-muted-foreground">{label}</dt>
        <dd className="truncate text-[11px] font-medium">{value}</dd>
      </div>
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
    <AccordionItem value={t.id} className="rounded-lg border border-border bg-card px-3 not-last:border-b-0">
      <AccordionPrimitive.Header className="flex items-center gap-1">
        <AccordionPrimitive.Trigger className="group/trigger flex flex-1 items-center justify-between gap-2 py-2.5 text-left outline-none">
          <div className="flex min-w-0 flex-col gap-1">
            <span className="inline-flex items-center gap-1 text-[10px] text-muted-foreground">
              <Server className="size-2.5" />
              {t.sshHost}
            </span>
            <div className="flex items-center gap-2">
              {t.running ? (
                <button
                  type="button"
                  title="Open in browser"
                  onClick={(e) => {
                    e.stopPropagation();
                    api.openInBrowser(t.localPort);
                  }}
                  className="truncate text-xs font-semibold hover:text-primary hover:underline"
                >
                  {t.localPort} → {t.remoteHost}:{t.remotePort}
                </button>
              ) : (
                <strong className="truncate text-xs font-semibold">
                  {t.localPort} → {t.remoteHost}:{t.remotePort}
                </strong>
              )}
              <span
                className={`inline-flex shrink-0 items-center gap-1 rounded-full px-1.5 py-0.5 text-[9px] font-medium ${statusPillClass(t)}`}
              >
                <span className={`size-1.5 rounded-full ${statusDotClass(t)}`} />
                {t.status === "retrying" ? `retrying (${t.retryAttempt}/6)` : statusTitle(t)}
              </span>
            </div>
            <div className="flex flex-wrap items-center gap-x-2.5 gap-y-0.5 text-[10px] text-muted-foreground">
              {t.running && t.latencyMs != null && t.status === "connected" && (
                <span className="inline-flex items-center gap-1">
                  <Gauge className="size-3" />
                  {t.latencyMs.toFixed(2)}ms
                </span>
              )}
              {t.status === "retrying" && (
                <span className="inline-flex items-center gap-1">
                  <Clock className="size-3" />
                  retry in {t.retryInSecs}s
                </span>
              )}
              {t.connectedSecs != null && (
                <span className="inline-flex items-center gap-1">
                  <Clock className="size-3" />
                  {fmtDuration(t.connectedSecs)}
                </span>
              )}
              {t.running && (t.bytesReceived != null || t.bytesSent != null) && (
                <>
                  <span className="inline-flex items-center gap-1" title="Downloaded">
                    <ArrowDown className="size-3" />
                    {fmtBytes(t.bytesReceived ?? 0)}
                  </span>
                  <span className="inline-flex items-center gap-1" title="Uploaded">
                    <ArrowUp className="size-3" />
                    {fmtBytes(t.bytesSent ?? 0)}
                  </span>
                </>
              )}
            </div>
          </div>
        </AccordionPrimitive.Trigger>

<div className="flex shrink-0 items-center gap-1">
            {t.running ? (
              <>
                <Button size="icon-sm" variant="ghost" title="Edit" onClick={onEdit}>
                  <Pencil className="size-4" />
                </Button>
                <Button size="icon-sm" variant="destructive" title="Stop" onClick={onStop}>
                  <Square className="size-4" />
                </Button>
              </>
            ) : (
            <>
              <Button size="icon-sm" variant="ghost" title="Start" onClick={onStart}>
                <Play className="size-4" />
              </Button>
              <Button size="icon-sm" variant="ghost" title="Edit" onClick={onEdit}>
                <Pencil className="size-4" />
              </Button>
              <Button size="icon-sm" variant="ghost" title="Delete" onClick={onDelete}>
                <Trash2 className="size-4" />
              </Button>
            </>
          )}
        </div>

        <AccordionPrimitive.Trigger className="group/trigger flex shrink-0 items-center py-2 outline-none">
          <ChevronDown className="size-4 shrink-0 text-muted-foreground group-aria-expanded/trigger:hidden" />
          <ChevronUp className="hidden size-4 shrink-0 text-muted-foreground group-aria-expanded/trigger:inline" />
        </AccordionPrimitive.Trigger>
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
              <dl className="grid grid-cols-2 gap-1.5">
                <DetailCell icon={Hash} label="PID" value={t.pid != null ? String(t.pid) : "—"} />
                <DetailCell
                  icon={ArrowLeftRight}
                  label="Local address"
                  value={`localhost:${t.localPort}`}
                />
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
