use tauri::tray::TrayIconBuilder;
use tauri::{App, Manager};

/// Builds the menu-bar tray icon. No menu is attached: macOS's tray-icon
/// backend shows the menu on *any* click regardless of
/// `show_menu_on_left_click`, which would swallow the toggle-window click.
/// Quitting is a button in the UI instead.
pub fn setup(app: &App) -> tauri::Result<()> {
    TrayIconBuilder::new()
        .icon(tauri::include_image!("icons/tray.png"))
        .icon_as_template(true)
        .on_tray_icon_event(|tray, event| {
            if let tauri::tray::TrayIconEvent::Click {
                button_state: tauri::tray::MouseButtonState::Up,
                ..
            } = event
            {
                let app = tray.app_handle();
                if let Some(window) = app.get_webview_window("main") {
                    if window.is_visible().unwrap_or(false) {
                        let _ = window.hide();
                    } else {
                        let _ = window.show();
                        let _ = window.set_focus();
                    }
                }
            }
        })
        .build(app)?;

    Ok(())
}
