export interface SshHost {
  alias: string;
  hostname: string | null;
  user: string | null;
  port: number | null;
}

export type TunnelStatus = "connecting" | "connected" | "retrying";

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
  /** Seconds the current connection has been up; null while not connected. */
  connectedSecs: number | null;
  pid: number | null;
  /** Set only while `status` is "retrying". */
  retryAttempt: number | null;
  retryInSecs: number | null;
  /** Cumulative bytes since this connection attempt started; null when not
   * running or when the platform has no way to sample it (e.g. Windows). */
  bytesReceived: number | null;
  bytesSent: number | null;
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

export interface HostCheck {
  reachable: boolean;
  latencyMs: number | null;
  error: string | null;
}
