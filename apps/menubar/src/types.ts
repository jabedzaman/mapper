export interface SshHost {
  alias: string;
  hostname: string | null;
  user: string | null;
  port: number | null;
}

export type TunnelStatus = "connecting" | "connected";

export interface TunnelInfo {
  id: string;
  sshHost: string;
  localPort: number;
  remoteHost: string;
  remotePort: number;
  pid: number;
  status: TunnelStatus;
  latencyMs: number | null;
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
