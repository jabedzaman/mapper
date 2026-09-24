use tauri::{AppHandle, Manager, State};

use crate::ssh_config::{self, SshHost};
use crate::tunnel::{self, OrphanedProcess, TunnelFailure, TunnelInfo, TunnelState};

#[tauri::command]
pub fn list_ssh_hosts() -> Vec<SshHost> {
    ssh_config::list_hosts()
}

#[tauri::command]
pub fn list_tunnels(state: State<TunnelState>) -> Vec<TunnelInfo> {
    state.list()
}

#[tauri::command]
pub fn start_tunnel(
    state: State<TunnelState>,
    ssh_host: String,
    local_port: u16,
    remote_host: String,
    remote_port: u16,
) -> Result<TunnelInfo, String> {
    state.start(ssh_host, local_port, remote_host, remote_port)
}

#[tauri::command]
pub fn stop_tunnel(state: State<TunnelState>, id: String) -> Result<(), String> {
    state.stop(&id)
}

#[tauri::command]
pub fn take_tunnel_failures(state: State<TunnelState>) -> Vec<TunnelFailure> {
    state.take_failures()
}

#[tauri::command]
pub fn kill_process_on_port(port: u16) -> Result<(), String> {
    tunnel::kill_process_on_port(port)
}

#[tauri::command]
pub fn get_tunnel_log(state: State<TunnelState>, id: String) -> Vec<String> {
    state.log(&id).unwrap_or_default()
}

#[tauri::command]
pub fn list_orphaned_ssh(state: State<TunnelState>) -> Vec<OrphanedProcess> {
    tunnel::list_orphaned_ssh(&state)
}

#[tauri::command]
pub fn kill_orphaned_ssh(pid: u32) -> Result<(), String> {
    tunnel::kill_orphaned_ssh(pid)
}

#[tauri::command]
pub fn quit_app(app: AppHandle) {
    app.state::<TunnelState>().stop_all();
    app.exit(0);
}
