import { invoke } from "@tauri-apps/api/core";
import type { OrphanedProcess, SshHost, Tunnel, TunnelFailure } from "../types";

export interface StartTunnelSpec {
  sshHost: string;
  localPort: number;
  remoteHost: string;
  remotePort: number;
  [key: string]: unknown;
}

export const api = {
  listSshHosts: () => invoke<SshHost[]>("list_ssh_hosts"),
  listTunnels: () => invoke<Tunnel[]>("list_tunnels"),
  startTunnel: (spec: StartTunnelSpec) => invoke<Tunnel>("start_tunnel", spec),
  startSavedTunnel: (id: string) => invoke<Tunnel>("start_saved_tunnel", { id }),
  stopTunnel: (id: string) => invoke<void>("stop_tunnel", { id }),
  deleteTunnel: (id: string) => invoke<void>("delete_tunnel", { id }),
  takeTunnelFailures: () => invoke<TunnelFailure[]>("take_tunnel_failures"),
  killProcessOnPort: (port: number) => invoke<void>("kill_process_on_port", { port }),
  getTunnelLog: (id: string) => invoke<string[]>("get_tunnel_log", { id }),
  listOrphanedSsh: () => invoke<OrphanedProcess[]>("list_orphaned_ssh"),
  killOrphanedSsh: (pid: number) => invoke<void>("kill_orphaned_ssh", { pid }),
  exportTunnels: (path: string) => invoke<void>("export_tunnels", { path }),
  importTunnels: (path: string) => invoke<number>("import_tunnels", { path }),
  quitApp: () => invoke<void>("quit_app"),
};
