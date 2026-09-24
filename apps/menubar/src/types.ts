export interface SshHost {
  alias: string;
  hostname: string | null;
  user: string | null;
  port: number | null;
}

export type TunnelStatus = "connecting" | "connected";

/**
 * A saved tunnel's persisted identity plus whatever live runtime state it
 * currently has. `running` reflects actual process state, reconciled by
 * the backend every poll — not just "the user asked for this once".
 */
export interface Tunnel {
  id: string;
  name: string;
  sshHost: string;
  localPort: number;
  remoteHost: string;
  remotePort: number;
  running: boolean;
  status: TunnelStatus | null;
  latencyMs: number | null;
  pid: number | null;
}

export interface TunnelFailure {
  id: string;
  sshHost: string;
  localPort: number;
  remoteHost: string;
  remotePort: number;
  message: string;
  portInUse: boolean;
}

export interface OrphanedProcess {
  pid: number;
  command: string;
}
