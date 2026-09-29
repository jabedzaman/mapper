import { useState } from "react";
import { Accordion as AccordionPrimitive } from "@base-ui/react/accordion";
import { ChevronDown, ChevronUp, Pencil, Plus, Trash2 } from "lucide-react";
import { Button } from "@/components/ui/button";
import { Accordion, AccordionItem, AccordionContent } from "@/components/ui/accordion";
import type { HostBlock } from "../types";

function DetailRow({ label, value }: { label: string; value: string }) {
  return (
    <div className="flex items-baseline justify-between gap-3">
      <dt className="shrink-0 text-muted-foreground">{label}</dt>
      <dd className="truncate font-medium">{value}</dd>
    </div>
  );
}

function summarize(h: HostBlock): string {
  const target = [h.user && `${h.user}@`, h.hostname || null].filter(Boolean).join("");
  const withPort = h.port ? `${target || "?"}:${h.port}` : target;
  return withPort || "no HostName set";
}

interface Props {
  hosts: HostBlock[];
  onEdit: (id: string) => void;
  onDelete: (id: string) => void;
  onAdd: () => void;
}

export function SshHostList({ hosts, onEdit, onDelete, onAdd }: Props) {
  const [openIds, setOpenIds] = useState<string[]>([]);

  return (
    <div className="flex flex-col gap-2.5">
      {hosts.length === 0 && (
        <p className="text-xs text-muted-foreground">No hosts yet. Add one below.</p>
      )}

      <Accordion value={openIds} onValueChange={(v) => setOpenIds(v as string[])} className="gap-2">
        {hosts.map((h) => (
          <AccordionItem
            key={h.id}
            value={h.id}
            className="rounded-lg border border-border bg-card px-3 not-last:border-b-0"
          >
            <AccordionPrimitive.Header className="flex items-center gap-1">
              <AccordionPrimitive.Trigger className="group/trigger flex flex-1 items-center justify-between gap-2 py-2.5 text-left outline-none">
                <div className="flex min-w-0 flex-col gap-0.5">
                  <strong className="truncate text-xs font-semibold">{h.patterns || "(no alias)"}</strong>
                  <span className="truncate text-[10px] text-muted-foreground">{summarize(h)}</span>
                </div>
              </AccordionPrimitive.Trigger>

              <div className="flex shrink-0 items-center gap-1">
                <Button size="icon-sm" variant="ghost" title="Edit" onClick={() => onEdit(h.id)}>
                  <Pencil className="size-4" />
                </Button>
                <Button size="icon-sm" variant="ghost" title="Delete" onClick={() => onDelete(h.id)}>
                  <Trash2 className="size-4" />
                </Button>
              </div>

              <AccordionPrimitive.Trigger className="group/trigger flex shrink-0 items-center py-2 outline-none">
                <ChevronDown className="size-4 shrink-0 text-muted-foreground group-aria-expanded/trigger:hidden" />
                <ChevronUp className="hidden size-4 shrink-0 text-muted-foreground group-aria-expanded/trigger:inline" />
              </AccordionPrimitive.Trigger>
            </AccordionPrimitive.Header>

            <AccordionContent>
              <dl className="flex flex-col gap-1 pb-1 text-[11px]">
                <DetailRow label="ProxyJump" value={h.proxyJump || "—"} />
                <DetailRow
                  label="IdentityFile"
                  value={h.identityFiles.split("\n").filter(Boolean).join(", ") || "—"}
                />
              </dl>
              {h.extra.trim() && (
                <pre className="mt-1 overflow-x-auto rounded-md bg-muted/50 p-2 text-[10px] whitespace-pre-wrap text-muted-foreground">
                  {h.extra}
                </pre>
              )}
            </AccordionContent>
          </AccordionItem>
        ))}
      </Accordion>

      <Button variant="outline" size="sm" className="gap-1.5 self-start" onClick={onAdd}>
        <Plus className="size-3.5" />
        Add host
      </Button>
    </div>
  );
}
