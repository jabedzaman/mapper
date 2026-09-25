use tauri::State;

use crate::tunnel::{self, OrphanedProcess, TunnelState};

#[tauri::command]
pub fn list_orphaned_ssh(state: State<TunnelState>) -> Vec<OrphanedProcess> {
    tunnel::list_orphaned_ssh(&state)
}

#[tauri::command]
pub fn kill_orphaned_ssh(pid: u32) -> Result<(), String> {
    tunnel::kill_orphaned_ssh(pid)
}

/// Opens the tunnel's forwarded local endpoint (`http://localhost:<port>`)
/// in the default browser.
#[tauri::command]
pub fn open_in_browser(local_port: u16) -> Result<(), String> {
    open_url(format!("http://localhost:{local_port}"))
}

/// Opens an arbitrary URL in the default browser — used for "Report a bug"
/// linking out to GitHub, but generic so it isn't tied to that one caller.
/// `open::that` already handles macOS/Windows/Linux on its own, so there's
/// no platform branching to do here.
#[tauri::command]
pub fn open_url(url: String) -> Result<(), String> {
    open::that(&url).map_err(|e| format!("failed to open url: {e}"))
}
