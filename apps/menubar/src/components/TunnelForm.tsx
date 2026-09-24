import { useState, type ReactNode } from "react";
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
    <form onSubmit={handleSubmit} className="form">
      <label>
        SSH config
        {hosts.length > 0 ? (
          <select value={sshHost} onChange={(e) => setSshHost(e.target.value)}>
            {hosts.map((h) => (
              <option key={h.alias} value={h.alias}>
                {h.alias}
                {h.hostname ? ` (${h.hostname})` : ""}
              </option>
            ))}
          </select>
        ) : (
          <input
            placeholder="user@host, or an alias from ~/.ssh/config"
            value={sshHost}
            onChange={(e) => setSshHost(e.target.value)}
          />
        )}
      </label>

      <div className="row">
        <label>
          Local port
          <input
            inputMode="numeric"
            placeholder="8080"
            value={localPort}
            onChange={(e) => setLocalPort(e.target.value)}
          />
        </label>
        <label>
          Remote port
          <input
            inputMode="numeric"
            placeholder="80"
            value={remotePort}
            onChange={(e) => setRemotePort(e.target.value)}
          />
        </label>
      </div>

      <label>
        Remote host
        <input
          placeholder="localhost"
          value={remoteHost}
          onChange={(e) => setRemoteHost(e.target.value)}
        />
      </label>

      {children}

      <button type="submit" disabled={starting}>
        {starting ? "Starting…" : "Start forwarding"}
      </button>
    </form>
  );
}
