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
import type { SshHost } from "../types";
import type { StartTunnelSpec } from "../lib/api";

interface Props {
  hosts: SshHost[];
  starting: boolean;
  onSubmit: (spec: StartTunnelSpec) => Promise<boolean>;
  onValidationError: (message: string) => void;
  /** Error / failure banners, rendered above the submit button. */
  children?: ReactNode;
}

export function TunnelForm({ hosts, starting, onSubmit, onValidationError, children }: Props) {
  const [sshHost, setSshHost] = useState(hosts[0]?.alias ?? "");
  const [localPort, setLocalPort] = useState("");
  const [remoteHost, setRemoteHost] = useState("localhost");
  const [remotePort, setRemotePort] = useState("");

  // Populate the default selection once the ssh config list loads.
  if (!sshHost && hosts.length > 0) {
    setSshHost(hosts[0].alias);
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
    if (ok) {
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
          <Select value={sshHost} onValueChange={(v) => setSshHost(v as string)}>
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
            onChange={(e) => setSshHost(e.target.value)}
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

      {children}

      <Button type="submit" disabled={starting} className="mt-1 w-full">
        {starting ? "Starting…" : "Start forwarding"}
      </Button>
    </form>
  );
}
