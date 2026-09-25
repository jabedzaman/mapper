import { useState } from "react";
import { Bug, ChevronLeft, Download, FileText, Power, Upload } from "lucide-react";
import { Button } from "@/components/ui/button";
import { Slider } from "@/components/ui/slider";
import { Switch } from "@/components/ui/switch";
import { Accordion, AccordionItem, AccordionTrigger, AccordionContent } from "@/components/ui/accordion";
import { api } from "../lib/api";
import { usePollInterval, MIN_POLL_INTERVAL_MS, MAX_POLL_INTERVAL_MS } from "../hooks/use-poll-interval";
import type { NotificationPermission } from "../hooks/use-app-preferences";

const GITHUB_REPO = "https://github.com/jabedzaman/mapper";

interface Props {
  onBack: () => void;
  onExport: () => void;
  onImport: () => void;
  canExport: boolean;
  onChangelog: () => void;
  error: string | null;
  showBadge: boolean;
  toggleBadge: (next: boolean) => void;
  launchAtLogin: boolean;
  toggleLaunchAtLogin: (next: boolean) => void;
  notifPermission: NotificationPermission;
  enableNotifications: () => void;
}

function SectionLabel({ children }: { children: string }) {
  return (
    <h2 className="px-1 text-[10px] font-semibold tracking-wide text-muted-foreground uppercase">
      {children}
    </h2>
  );
}

function SettingsRow({
  icon,
  label,
  description,
  onClick,
  disabled,
}: {
  icon: React.ReactNode;
  label: string;
  description?: string;
  onClick: () => void;
  disabled?: boolean;
}) {
  return (
    <button
      type="button"
      className="flex w-full items-center gap-2.5 border-t border-border px-3 py-2.5 text-left transition-colors first:border-t-0 hover:bg-muted/50 disabled:cursor-not-allowed disabled:opacity-50 disabled:hover:bg-transparent"
      onClick={onClick}
      disabled={disabled}
    >
      <span className="shrink-0 text-muted-foreground">{icon}</span>
      <div className="min-w-0 flex-1">
        <p className="text-xs font-medium">{label}</p>
        {description && <p className="mt-0.5 text-[10px] text-muted-foreground">{description}</p>}
      </div>
    </button>
  );
}

export function SettingsPage({
  onBack,
  onExport,
  onImport,
  canExport,
  onChangelog,
  error,
  showBadge,
  toggleBadge,
  launchAtLogin,
  toggleLaunchAtLogin,
  notifPermission,
  enableNotifications,
}: Props) {
  const { intervalMs, setIntervalMs } = usePollInterval();
  const seconds = Math.round(intervalMs / 1000);
  const [intervalOpen, setIntervalOpen] = useState<string[]>([]);

  function reportBug() {
    const title = encodeURIComponent("bug: ");
    api.openUrl(`${GITHUB_REPO}/issues/new?title=${title}&labels=bug`);
  }

  return (
    <div className="flex min-h-0 flex-1 flex-col">
      <div className="px-2 py-2.5">
        <Button variant="ghost" size="icon-sm" title="Back" onClick={onBack}>
          <ChevronLeft className="size-4" />
        </Button>
      </div>

      <div className="flex min-h-0 flex-1 flex-col gap-5 overflow-y-auto px-4 pb-4">
        <div className="flex flex-col gap-1.5">
          <SectionLabel>General</SectionLabel>
          <div className="overflow-hidden rounded-xl border border-border bg-card">
            <div className="flex items-center justify-between gap-3 px-3 py-2.5">
              <div className="min-w-0 flex-1">
                <p className="text-xs font-medium">Tray badge</p>
                <p className="mt-0.5 text-[10px] text-muted-foreground">
                  Show the active tunnel count next to the menu bar icon.
                </p>
              </div>
              <Switch checked={showBadge} onCheckedChange={toggleBadge} />
            </div>

            <div className="flex items-center justify-between gap-3 border-t border-border px-3 py-2.5">
              <div className="min-w-0 flex-1">
                <p className="text-xs font-medium">Launch at login</p>
                <p className="mt-0.5 text-[10px] text-muted-foreground">
                  Start Mapper automatically when you log in.
                </p>
              </div>
              <Switch checked={launchAtLogin} onCheckedChange={toggleLaunchAtLogin} />
            </div>

            {notifPermission !== "granted" && (
              <div className="flex items-center justify-between gap-3 border-t border-border px-3 py-2.5">
                <div className="min-w-0 flex-1">
                  <p className="text-xs font-medium">Notifications</p>
                  <p className="mt-0.5 text-[10px] text-muted-foreground">
                    Alert when a tunnel gives up reconnecting for good.
                  </p>
                </div>
                <Button variant="secondary" size="sm" onClick={enableNotifications}>
                  Enable
                </Button>
              </div>
            )}

            <Accordion
              value={intervalOpen}
              onValueChange={(v) => setIntervalOpen(v as string[])}
              className="border-t border-border"
            >
              <AccordionItem value="poll">
                <AccordionTrigger className="gap-2 px-3">
                  <span className="flex-1 text-xs font-medium">Ping poll interval</span>
                  <span className="text-xs text-muted-foreground">{seconds}s</span>
                </AccordionTrigger>
                <AccordionContent className="text-xs">
                  <div className="px-3 pt-1">
                    <Slider
                      value={[seconds]}
                      min={MIN_POLL_INTERVAL_MS / 1000}
                      max={MAX_POLL_INTERVAL_MS / 1000}
                      step={5}
                      onValueChange={(v) => setIntervalMs((Array.isArray(v) ? v[0] : v) * 1000)}
                    />
                    <p className="mt-2 text-[10px] text-muted-foreground">
                      How often tunnel status, latency, and the New Tunnel host check refresh.
                      Lower is more real-time; higher uses less CPU.
                    </p>
                  </div>
                </AccordionContent>
              </AccordionItem>
            </Accordion>
          </div>
        </div>

        <div className="flex flex-col gap-1.5">
          <SectionLabel>Your tunnels</SectionLabel>
          <div className="overflow-hidden rounded-xl border border-border bg-card">
            <SettingsRow
              icon={<Download className="size-4" />}
              label="Export tunnels"
              description="Save your saved forwards to a JSON file to share or back up."
              onClick={onExport}
              disabled={!canExport}
            />
            <SettingsRow
              icon={<Upload className="size-4" />}
              label="Import tunnels"
              description="Merge forwards from a JSON file exported by Mapper. Always imports stopped."
              onClick={onImport}
            />
          </div>
          {error && <p className="px-1 text-xs text-destructive">{error}</p>}
        </div>

        <div className="flex flex-col gap-1.5">
          <SectionLabel>Help</SectionLabel>
          <div className="overflow-hidden rounded-xl border border-border bg-card">
            <SettingsRow
              icon={<FileText className="size-4" />}
              label="Changelog"
              description="What's new in this version of Mapper."
              onClick={onChangelog}
            />
            <SettingsRow
              icon={<Bug className="size-4" />}
              label="Report a bug"
              description="Opens a pre-filled GitHub issue for this repo."
              onClick={reportBug}
            />
          </div>
        </div>

        <button
          type="button"
          className="flex items-center justify-center gap-1.5 rounded-lg py-2 text-xs font-medium text-destructive transition-colors hover:bg-destructive/10"
          onClick={() => api.quitApp()}
        >
          <Power className="size-3.5" />
          Quit Mapper
        </button>
      </div>
    </div>
  );
}
