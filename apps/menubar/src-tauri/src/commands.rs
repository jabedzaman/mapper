use std::collections::{HashMap, HashSet};

use serde::Serialize;
use tauri::{AppHandle, Manager, State};

use crate::saved_tunnels::SavedTunnelStore;
use crate::ssh_config::{self, SshHost};
use crate::tunnel::{self, OrphanedProcess, TunnelFailure, TunnelInfo, TunnelState, TunnelStatus};

/// The merged view the frontend actually renders: a saved tunnel's
/// persisted identity plus whatever live runtime state it currently has,
/// if it's running.
#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct TunnelView {
    pub id: String,
    pub name: String,
    pub ssh_host: String,
    pub local_port: u16,
    pub remote_host: String,
    pub remote_port: u16,
    pub running: bool,
    pub status: Option<TunnelStatus>,
    pub latency_ms: Option<f64>,
    pub pid: Option<u32>,
    pub retry_attempt: Option<u32>,
    pub retry_in_secs: Option<u64>,
}

#[tauri::command]
pub fn list_ssh_hosts() -> Vec<SshHost> {
    ssh_config::list_hosts()
}

#[tauri::command]
pub fn list_tunnels(state: State<TunnelState>, saved: State<SavedTunnelStore>) -> Vec<TunnelView> {
    let live = state.list();
    let live_ids: HashSet<String> = live.iter().map(|t| t.id.clone()).collect();
    // A saved tunnel marked running whose process isn't actually alive
    // died (or a start attempt never launched) — reflect that on disk too.
    saved.reconcile(&live_ids);

    let mut live_by_id: HashMap<String, TunnelInfo> =
        live.into_iter().map(|t| (t.id.clone(), t)).collect();

    saved
        .list()
        .into_iter()
        .map(|t| {
            let live = live_by_id.remove(&t.id);
            TunnelView {
                id: t.id,
                name: t.name,
                ssh_host: t.ssh_host,
                local_port: t.local_port,
                remote_host: t.remote_host,
                remote_port: t.remote_port,
                running: live.is_some(),
                status: live.as_ref().map(|l| l.status.clone()),
                latency_ms: live.as_ref().and_then(|l| l.latency_ms),
                pid: live.as_ref().and_then(|l| l.pid),
                retry_attempt: live.as_ref().and_then(|l| l.retry_attempt),
                retry_in_secs: live.as_ref().and_then(|l| l.retry_in_secs),
            }
        })
        .collect()
}

#[tauri::command]
pub fn start_tunnel(
    state: State<TunnelState>,
    saved: State<SavedTunnelStore>,
    ssh_host: String,
    local_port: u16,
    remote_host: String,
    remote_port: u16,
) -> Result<TunnelView, String> {
    let saved_tunnel = saved.start(&ssh_host, local_port, &remote_host, remote_port);

    match state.start(
        saved_tunnel.id.clone(),
        ssh_host,
        local_port,
        remote_host,
        remote_port,
    ) {
        Ok(info) => Ok(TunnelView {
            id: saved_tunnel.id,
            name: saved_tunnel.name,
            ssh_host: info.ssh_host,
            local_port: info.local_port,
            remote_host: info.remote_host,
            remote_port: info.remote_port,
            running: true,
            status: Some(info.status),
            latency_ms: info.latency_ms,
            pid: info.pid,
            retry_attempt: info.retry_attempt,
            retry_in_secs: info.retry_in_secs,
        }),
        Err(e) => {
            // Never actually launched — don't leave it persisted as running.
            let _ = saved.set_running(&saved_tunnel.id, false);
            Err(e)
        }
    }
}

/// Starts a specific already-saved (currently stopped) tunnel by id.
#[tauri::command]
pub fn start_saved_tunnel(
    state: State<TunnelState>,
    saved: State<SavedTunnelStore>,
    id: String,
) -> Result<TunnelView, String> {
    let Some(t) = saved.get(&id) else {
        return Err(format!("no saved tunnel with id {id}"));
    };
    start_tunnel(
        state,
        saved,
        t.ssh_host,
        t.local_port,
        t.remote_host,
        t.remote_port,
    )
}

#[tauri::command]
pub fn stop_tunnel(
    state: State<TunnelState>,
    saved: State<SavedTunnelStore>,
    id: String,
) -> Result<(), String> {
    state.stop(&id)?;
    saved.set_running(&id, false)
}

#[tauri::command]
pub fn delete_tunnel(
    state: State<TunnelState>,
    saved: State<SavedTunnelStore>,
    id: String,
) -> Result<(), String> {
    let _ = state.stop(&id); // best-effort: fine if it wasn't running
    saved.remove(&id)
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
pub fn export_tunnels(saved: State<SavedTunnelStore>, path: String) -> Result<(), String> {
    saved.export_to(&path)
}

#[tauri::command]
pub fn import_tunnels(saved: State<SavedTunnelStore>, path: String) -> Result<usize, String> {
    saved.import_from(&path)
}

#[tauri::command]
pub fn quit_app(app: AppHandle) {
    app.state::<TunnelState>().stop_all();
    app.exit(0);
}
