use tauri::State;

use crate::saved_tunnels::SavedTunnelStore;

#[tauri::command]
pub fn export_tunnels(saved: State<SavedTunnelStore>, path: String) -> Result<(), String> {
    saved.export_to(&path)
}

#[tauri::command]
pub fn import_tunnels(saved: State<SavedTunnelStore>, path: String) -> Result<usize, String> {
    saved.import_from(&path)
}
