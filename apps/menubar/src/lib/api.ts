import { invoke } from "@tauri-apps/api/core";
import type { OrphanedProcess, SshHost, TunnelFailure, TunnelInfo } from "../types";

export interface StartTunnelSpec {
  sshHost: string;
  localPort: number;
  remoteHost: string;
  remotePort: number;
  [key: string]: unknown;
}

export const api = {
  listSshHosts: () => invoke<SshHost[]>("list_ssh_hosts"),
  listTunnels: () => invoke<TunnelInfo[]>("list_tunnels"),
  takeTunnelFailures: () => invoke<TunnelFailure[]>("take_tunnel_failures"),
  startTunnel: (spec: StartTunnelSpec) => invoke<TunnelInfo>("start_tunnel", spec),
  stopTunnel: (id: string) => invoke<void>("stop_tunnel", { id }),
  killProcessOnPort: (port: number) => invoke<void>("kill_process_on_port", { port }),
  getTunnelLog: (id: string) => invoke<string[]>("get_tunnel_log", { id }),
  listOrphanedSsh: () => invoke<OrphanedProcess[]>("list_orphaned_ssh"),
  killOrphanedSsh: (pid: number) => invoke<void>("kill_orphaned_ssh", { pid }),
  quitApp: () => invoke<void>("quit_app"),
};
