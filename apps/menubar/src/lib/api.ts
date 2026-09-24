import { invoke } from "@tauri-apps/api/core";
import type { HostCheck, OrphanedProcess, SshHost, Tunnel, TunnelFailure } from "../types";

export interface StartTunnelSpec {
  sshHost: string;
  localPort: number;
  remoteHost: string;
  remotePort: number;
  [key: string]: unknown;
}

export const api = {
  listSshHosts: () => invoke<SshHost[]>("list_ssh_hosts"),
  checkSshHost: (alias: string) => invoke<HostCheck>("check_ssh_host", { alias }),
  listTunnels: () => invoke<Tunnel[]>("list_tunnels"),
  startTunnel: (spec: StartTunnelSpec) => invoke<Tunnel>("start_tunnel", spec),
  startSavedTunnel: (id: string) => invoke<Tunnel>("start_saved_tunnel", { id }),
  stopTunnel: (id: string) => invoke<void>("stop_tunnel", { id }),
  updateTunnel: (id: string, spec: StartTunnelSpec) =>
    invoke<Tunnel>("update_tunnel", { id, ...spec }),
  deleteTunnel: (id: string) => invoke<void>("delete_tunnel", { id }),
  takeTunnelFailures: () => invoke<TunnelFailure[]>("take_tunnel_failures"),
  killProcessOnPort: (port: number) => invoke<void>("kill_process_on_port", { port }),
  getPortOwners: (port: number) => invoke<OrphanedProcess[]>("get_port_owners", { port }),
  openInBrowser: (localPort: number) => invoke<void>("open_in_browser", { localPort }),
  openUrl: (url: string) => invoke<void>("open_url", { url }),
  getTunnelLog: (id: string) => invoke<string[]>("get_tunnel_log", { id }),
  listOrphanedSsh: () => invoke<OrphanedProcess[]>("list_orphaned_ssh"),
  killOrphanedSsh: (pid: number) => invoke<void>("kill_orphaned_ssh", { pid }),
  exportTunnels: (path: string) => invoke<void>("export_tunnels", { path }),
  importTunnels: (path: string) => invoke<number>("import_tunnels", { path }),
  getChangelog: () => invoke<string>("get_changelog"),
  getShowTrayBadge: () => invoke<boolean>("get_show_tray_badge"),
  setShowTrayBadge: (show: boolean) => invoke<void>("set_show_tray_badge", { show }),
  quitApp: () => invoke<void>("quit_app"),
};
