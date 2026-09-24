export interface SshHost {
  alias: string;
  hostname: string | null;
  user: string | null;
  port: number | null;
}

export interface TunnelInfo {
  id: string;
  sshHost: string;
  localPort: number;
  remoteHost: string;
  remotePort: number;
  pid: number;
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
