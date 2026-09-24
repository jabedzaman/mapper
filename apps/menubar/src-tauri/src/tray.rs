use std::sync::Mutex;

use tauri::menu::{CheckMenuItem, Menu, MenuBuilder, MenuEvent, MenuItem};
use tauri::tray::{TrayIcon, TrayIconBuilder};
use tauri::{App, AppHandle, Manager};

use crate::commands;
use crate::commands::TunnelView;
use crate::saved_tunnels::SavedTunnelStore;
use crate::tunnel::{TunnelState, TunnelStatus};

const OPEN_ID: &str = "open";
const QUIT_ID: &str = "quit";

/// What the menu was last built from, so `refresh_menu` can skip
/// rebuilding when nothing actually changed. Necessary, not just an
/// optimization: replacing a status item's menu object while it's open is
/// what makes macOS immediately dismiss it, so rebuilding on every poll
/// tick made the dropdown close on the user almost as soon as they opened
/// it.
static LAST_SIGNATURE: Mutex<Option<String>> = Mutex::new(None);

/// Builds the menu-bar tray icon with an attached dropdown menu. macOS
/// shows an attached menu on *any* click regardless of
/// `show_menu_on_left_click`, so there's no way to keep the old
/// left-click-toggles-the-popover behavior once a menu exists — the menu
/// itself carries an "Open Mapper" item to get to the full window instead.
pub fn setup(app: &App) -> tauri::Result<TrayIcon> {
    let menu = build_menu(app.handle(), &[])?;

    TrayIconBuilder::new()
        .icon(tauri::include_image!("icons/tray.png"))
        .icon_as_template(true)
        .menu(&menu)
        .on_menu_event(handle_menu_event)
        .build(app)
}

fn handle_menu_event(app: &AppHandle, event: MenuEvent) {
    match event.id().as_ref() {
        OPEN_ID => {
            if let Some(window) = app.get_webview_window("main") {
                let _ = window.show();
                let _ = window.set_focus();
            }
        }
        QUIT_ID => {
            app.state::<TunnelState>().stop_all();
            app.exit(0);
        }
        id => toggle_tunnel(app, id),
    }
}

/// A tunnel row's checkbox was clicked: flip it via the same start/stop
/// paths the popover uses, then rebuild the menu immediately so the
/// checkbox doesn't wait for the frontend's next poll to catch up.
fn toggle_tunnel(app: &AppHandle, id: &str) {
    let Some(saved) = app.state::<SavedTunnelStore>().get(id) else {
        return;
    };

    if saved.running {
        let _ = commands::stop_tunnel(app.state::<TunnelState>(), app.state::<SavedTunnelStore>(), id.to_string());
    } else {
        let _ = commands::start_saved_tunnel(
            app.state::<TunnelState>(),
            app.state::<SavedTunnelStore>(),
            id.to_string(),
        );
    }

    // Reuses list_tunnels' own tray refresh rather than duplicating it here.
    let _ = commands::list_tunnels(app.clone(), app.state::<TunnelState>(), app.state::<SavedTunnelStore>());
}

/// ● connected, ◐ connecting/retrying, ○ stopped — a glance is the point of
/// a quick view, so status is a glyph rather than a text column.
fn status_glyph(t: &TunnelView) -> &'static str {
    if !t.running {
        return "○";
    }
    match t.status {
        Some(TunnelStatus::Connected) => "●",
        _ => "◐",
    }
}

/// The port/host summary is what distinguishes two tunnels through the
/// same ssh host; the status glyph carries the finer-grained
/// connecting/retrying/connected state the checkbox alone can't show.
fn menu_label(t: &TunnelView) -> String {
    format!("{} {} \u{2192} {}:{}", status_glyph(t), t.local_port, t.remote_host, t.remote_port)
}

/// A disabled, unchecked item used purely as a section heading — native
/// menus have no dedicated header widget, so this is the usual stand-in.
fn section_header(app: &AppHandle, id: &str, label: &str) -> tauri::Result<MenuItem<tauri::Wry>> {
    MenuItem::with_id(app, id, label, false, None::<&str>)
}

fn build_menu(app: &AppHandle, views: &[TunnelView]) -> tauri::Result<Menu<tauri::Wry>> {
    let mut builder = MenuBuilder::new(app);

    if views.is_empty() {
        builder = builder.item(&section_header(app, "empty", "No tunnels")?);
    } else {
        let (running, stopped): (Vec<_>, Vec<_>) = views.iter().partition(|t| t.running);

        if !running.is_empty() {
            builder = builder.item(&section_header(app, "hdr-running", "Running")?);
            for t in &running {
                builder = builder.item(&CheckMenuItem::with_id(app, &t.id, menu_label(t), true, true, None::<&str>)?);
            }
        }

        if !stopped.is_empty() {
            if !running.is_empty() {
                builder = builder.separator();
            }
            builder = builder.item(&section_header(app, "hdr-stopped", "Stopped")?);
            for t in &stopped {
                builder = builder.item(&CheckMenuItem::with_id(app, &t.id, menu_label(t), true, false, None::<&str>)?);
            }
        }
    }

    builder
        .separator()
        .text(OPEN_ID, "Open Mapper")
        .text(QUIT_ID, "Quit Mapper")
        .build()
}

/// What a menu built from these views would actually show — running set,
/// order, and each row's checked state and label. Latency and retry
/// countdowns aren't included, so a tick that only moves those numbers
/// doesn't count as a change.
fn menu_signature(views: &[TunnelView]) -> String {
    views
        .iter()
        .map(|t| format!("{}|{}", t.id, menu_label(t)))
        .collect::<Vec<_>>()
        .join("\u{1}")
}

/// Rebuilds and re-attaches the tray's dropdown menu from the current
/// merged tunnel views, called on every `list_tunnels` poll from the
/// frontend since native menu content isn't reactive on its own. Skips
/// the rebuild when nothing actually changed — seeing this called
/// unconditionally used to close the menu on the user mid-click almost
/// every time, since a poll tick with the menu open was common enough to
/// hit constantly. Best-effort: a failure here shouldn't break the
/// tunnel list response.
pub fn refresh_menu(app: &AppHandle, tray: &TrayIcon, views: &[TunnelView]) {
    let signature = menu_signature(views);
    let mut last = LAST_SIGNATURE.lock().unwrap();
    if last.as_deref() == Some(signature.as_str()) {
        return;
    }

    if let Ok(menu) = build_menu(app, views) {
        let _ = tray.set_menu(Some(menu));
        *last = Some(signature);
    }
}
