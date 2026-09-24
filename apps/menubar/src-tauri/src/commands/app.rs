use tauri::{AppHandle, Manager, State};

use crate::settings::SettingsStore;
use crate::tunnel::TunnelState;

#[tauri::command]
pub fn quit_app(app: AppHandle) {
    app.state::<TunnelState>().stop_all();
    app.exit(0);
}

/// Embedded at compile time so the changelog is always available offline,
/// with no bundled resource file or runtime path to get wrong.
#[tauri::command]
pub fn get_changelog() -> &'static str {
    include_str!("../../../../../CHANGELOG.md")
}

#[tauri::command]
pub fn get_show_tray_badge(settings: State<SettingsStore>) -> bool {
    settings.show_tray_badge()
}

#[tauri::command]
pub fn set_show_tray_badge(
    tunnel_state: State<TunnelState>,
    settings: State<SettingsStore>,
    show: bool,
) -> Result<(), String> {
    settings.set_show_tray_badge(show)?;
    tunnel_state.set_badge_visible(show);
    Ok(())
}
