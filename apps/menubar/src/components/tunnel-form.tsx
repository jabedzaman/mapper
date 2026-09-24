import { useState, type ReactNode } from "react";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";
import { useHostCheck } from "../hooks/use-host-check";
import type { SshHost } from "../types";
import type { StartTunnelSpec } from "../lib/api";

const LAST_HOST_KEY = "mapper.lastSshHost";

function readLastHost(): string {
  try {
    return localStorage.getItem(LAST_HOST_KEY) ?? "";
  } catch {
    return "";
  }
}

function rememberHost(alias: string) {
  try {
    localStorage.setItem(LAST_HOST_KEY, alias);
  } catch {
    // best-effort only — a private window or blocked storage just means
    // the dropdown won't remember next launch, nothing else breaks.
  }
}

interface Props {
  hosts: SshHost[];
  starting: boolean;
  onSubmit: (spec: StartTunnelSpec) => Promise<boolean>;
  onValidationError: (message: string) => void;
  /** Pre-fills the form for editing an existing tunnel instead of creating one. */
  initial?: StartTunnelSpec;
  /** Submit button label; defaults to "Start forwarding". */
  submitLabel?: string;
  submittingLabel?: string;
  /** Error / failure banners, rendered above the submit button. */
  children?: ReactNode;
}

export function TunnelForm({
  hosts,
  starting,
  onSubmit,
  onValidationError,
  initial,
  submitLabel = "Start forwarding",
  submittingLabel = "Starting…",
  children,
}: Props) {
  const [sshHost, setSshHost] = useState(() => initial?.sshHost || readLastHost() || hosts[0]?.alias || "");
  const [localPort, setLocalPort] = useState(() => (initial ? String(initial.localPort) : ""));
  const [remoteHost, setRemoteHost] = useState(initial?.remoteHost ?? "localhost");
  const [remotePort, setRemotePort] = useState(() => (initial ? String(initial.remotePort) : ""));
  const { check: hostCheck, checking: hostChecking } = useHostCheck(sshHost);

  // Populate the default selection once the ssh config list loads, if we
  // didn't already restore one from last time.
  if (!sshHost && hosts.length > 0) {
    setSshHost(hosts[0].alias);
  }

  function selectHost(alias: string) {
    setSshHost(alias);
    rememberHost(alias);
  }

  async function handleSubmit(e: React.FormEvent) {
    e.preventDefault();
    const lp = Number(localPort);
    const rp = Number(remotePort);
    if (!sshHost || !lp || !rp) {
      onValidationError("fill in host, local port and remote port");
      return;
    }
    const ok = await onSubmit({
      sshHost,
      localPort: lp,
      remoteHost: remoteHost || "localhost",
      remotePort: rp,
    });
    if (ok && !initial) {
      setLocalPort("");
      setRemotePort("");
    }
  }

  return (
    <form onSubmit={handleSubmit} className="flex flex-col gap-3 border-b border-border p-4">
      <div className="flex flex-col gap-1.5">
        <Label htmlFor="ssh-host" className="text-xs text-muted-foreground">
          SSH config
        </Label>
        {hosts.length > 0 ? (
          <Select value={sshHost} onValueChange={(v) => selectHost(v as string)}>
            <SelectTrigger id="ssh-host" className="w-full">
              <SelectValue placeholder="Select a host" />
            </SelectTrigger>
            <SelectContent>
              {hosts.map((h) => (
                <SelectItem key={h.alias} value={h.alias}>
                  {h.alias}
                  {h.hostname ? ` (${h.hostname})` : ""}
                </SelectItem>
              ))}
            </SelectContent>
          </Select>
        ) : (
          <Input
            id="ssh-host"
            placeholder="user@host, or an alias from ~/.ssh/config"
            value={sshHost}
            onChange={(e) => selectHost(e.target.value)}
          />
        )}
      </div>

      <div className="flex gap-2.5">
        <div className="flex flex-1 flex-col gap-1.5">
          <Label htmlFor="local-port" className="text-xs text-muted-foreground">
            Local port
          </Label>
          <Input
            id="local-port"
            inputMode="numeric"
            placeholder="8080"
            value={localPort}
            onChange={(e) => setLocalPort(e.target.value)}
          />
        </div>
        <div className="flex flex-1 flex-col gap-1.5">
          <Label htmlFor="remote-port" className="text-xs text-muted-foreground">
            Remote port
          </Label>
          <Input
            id="remote-port"
            inputMode="numeric"
            placeholder="80"
            value={remotePort}
            onChange={(e) => setRemotePort(e.target.value)}
          />
        </div>
      </div>

      <div className="flex flex-col gap-1.5">
        <Label htmlFor="remote-host" className="text-xs text-muted-foreground">
          Remote host
        </Label>
        <Input
          id="remote-host"
          placeholder="localhost"
          value={remoteHost}
          onChange={(e) => setRemoteHost(e.target.value)}
        />
      </div>

      {sshHost.trim() && (
        <div className="flex items-center gap-1.5 text-[10px] text-muted-foreground">
          <span
            className={`size-1.5 shrink-0 rounded-full ${
              hostChecking && !hostCheck
                ? "animate-pulse bg-muted-foreground/40"
                : hostCheck?.reachable
                  ? "bg-emerald-500"
                  : "animate-pulse bg-red-500"
            }`}
          />
          {hostChecking && !hostCheck
            ? "Checking connection…"
            : hostCheck?.reachable
              ? `Reachable · ${hostCheck.latencyMs?.toFixed(2)}ms`
              : `Unreachable${hostCheck?.error ? ` · ${hostCheck.error}` : ""}`}
        </div>
      )}

      {children}

      <Button type="submit" disabled={starting} className="mt-1 w-full">
        {starting ? submittingLabel : submitLabel}
      </Button>
    </form>
  );
}
