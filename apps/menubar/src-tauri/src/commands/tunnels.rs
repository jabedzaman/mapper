use std::collections::{HashMap, HashSet};

use serde::Serialize;
use tauri::{AppHandle, State};

use crate::saved_tunnels::SavedTunnelStore;
use crate::tunnel::{self, TunnelFailure, TunnelInfo, TunnelState, TunnelStatus};

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
    /// Seconds the current connection has been up.
    pub connected_secs: Option<u64>,
    pub pid: Option<u32>,
    pub retry_attempt: Option<u32>,
    pub retry_in_secs: Option<u64>,
    pub bytes_received: Option<u64>,
    pub bytes_sent: Option<u64>,
}

#[tauri::command]
pub fn list_tunnels(
    app: AppHandle,
    state: State<TunnelState>,
    saved: State<SavedTunnelStore>,
) -> Vec<TunnelView> {
    let live = state.list();
    let live_ids: HashSet<String> = live.iter().map(|t| t.id.clone()).collect();
    // A saved tunnel marked running whose process isn't actually alive
    // died (or a start attempt never launched) — reflect that on disk too.
    saved.reconcile(&live_ids);

    let mut live_by_id: HashMap<String, TunnelInfo> =
        live.into_iter().map(|t| (t.id.clone(), t)).collect();

    let views: Vec<TunnelView> = saved
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
                connected_secs: live.as_ref().and_then(|l| l.connected_secs),
                pid: live.as_ref().and_then(|l| l.pid),
                retry_attempt: live.as_ref().and_then(|l| l.retry_attempt),
                retry_in_secs: live.as_ref().and_then(|l| l.retry_in_secs),
                bytes_received: live.as_ref().and_then(|l| l.bytes_received),
                bytes_sent: live.as_ref().and_then(|l| l.bytes_sent),
            }
        })
        .collect();

    // Piggybacks on the frontend's existing poll loop rather than running a
    // separate timer just to keep the tray dropdown's contents current.
    if let Some(tray) = state.tray() {
        crate::tray::refresh_menu(&app, &tray, &views);
    }

    views
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
            connected_secs: info.connected_secs,
            pid: info.pid,
            retry_attempt: info.retry_attempt,
            retry_in_secs: info.retry_in_secs,
            bytes_received: info.bytes_received,
            bytes_sent: info.bytes_sent,
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

/// Edits a saved tunnel's connection in place. If it was running, the old
/// process is stopped and a new one launched with the updated params under
/// the same id — otherwise just the saved record changes.
#[tauri::command]
pub fn update_tunnel(
    state: State<TunnelState>,
    saved: State<SavedTunnelStore>,
    id: String,
    ssh_host: String,
    local_port: u16,
    remote_host: String,
    remote_port: u16,
) -> Result<TunnelView, String> {
    let was_running = saved.get(&id).map(|t| t.running).unwrap_or(false);
    if was_running {
        let _ = state.stop(&id);
    }

    let updated = saved.update(&id, &ssh_host, local_port, &remote_host, remote_port)?;

    if !was_running {
        return Ok(TunnelView {
            id: updated.id,
            name: updated.name,
            ssh_host: updated.ssh_host,
            local_port: updated.local_port,
            remote_host: updated.remote_host,
            remote_port: updated.remote_port,
            running: false,
            status: None,
            latency_ms: None,
            connected_secs: None,
            pid: None,
            retry_attempt: None,
            retry_in_secs: None,
            bytes_received: None,
            bytes_sent: None,
        });
    }

    match state.start(updated.id.clone(), ssh_host, local_port, remote_host, remote_port) {
        Ok(info) => Ok(TunnelView {
            id: updated.id,
            name: updated.name,
            ssh_host: info.ssh_host,
            local_port: info.local_port,
            remote_host: info.remote_host,
            remote_port: info.remote_port,
            running: true,
            status: Some(info.status),
            latency_ms: info.latency_ms,
            connected_secs: info.connected_secs,
            pid: info.pid,
            retry_attempt: info.retry_attempt,
            retry_in_secs: info.retry_in_secs,
            bytes_received: info.bytes_received,
            bytes_sent: info.bytes_sent,
        }),
        Err(e) => {
            let _ = saved.set_running(&updated.id, false);
            Err(e)
        }
    }
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

/// What's currently listening on `port`, so the UI can show it before the
/// user confirms killing it.
#[tauri::command]
pub fn get_port_owners(port: u16) -> Vec<tunnel::OrphanedProcess> {
    tunnel::get_port_owners(port)
}

#[tauri::command]
pub fn get_tunnel_log(state: State<TunnelState>, id: String) -> Vec<String> {
    state.log(&id).unwrap_or_default()
}
