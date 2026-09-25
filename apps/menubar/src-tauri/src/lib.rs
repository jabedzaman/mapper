mod commands;
mod platform;
mod saved_tunnels;
mod settings;
mod shutdown;
mod ssh_config;
mod tray;
mod tunnel;

use tauri::{App, Manager};
use saved_tunnels::SavedTunnelStore;
use settings::SettingsStore;
use tunnel::TunnelState;

/// Re-launches every saved tunnel that was running last time the app
/// closed — the whole point of persisting `running` at all.
fn restart_saved_tunnels(app: &App) {
    let saved = app.state::<SavedTunnelStore>();
    let runtime = app.state::<TunnelState>();

    for t in saved.list() {
        if !t.running {
            continue;
        }
        if runtime
            .start(t.id.clone(), t.ssh_host, t.local_port, t.remote_host, t.remote_port)
            .is_err()
        {
            let _ = saved.set_running(&t.id, false);
        }
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_autostart::init(
            tauri_plugin_autostart::MacosLauncher::LaunchAgent,
            None,
        ))
        .manage(TunnelState::default())
        .invoke_handler(tauri::generate_handler![
            commands::list_ssh_hosts,
            commands::check_ssh_host,
            commands::list_tunnels,
            commands::start_tunnel,
            commands::start_saved_tunnel,
            commands::stop_tunnel,
            commands::update_tunnel,
            commands::delete_tunnel,
            commands::take_tunnel_failures,
            commands::kill_process_on_port,
            commands::get_port_owners,
            commands::open_in_browser,
            commands::open_url,
            commands::get_tunnel_log,
            commands::list_orphaned_ssh,
            commands::kill_orphaned_ssh,
            commands::export_tunnels,
            commands::import_tunnels,
            commands::quit_app,
            commands::get_show_tray_badge,
            commands::set_show_tray_badge,
            commands::get_changelog,
        ])
        .setup(|app| {
            // Menu-bar-only app: no Dock icon, no app menu bar.
            #[cfg(target_os = "macos")]
            app.set_activation_policy(tauri::ActivationPolicy::Accessory);

            let tray = tray::setup(app)?;
            settings::setup(app)?;
            let show_badge = app.state::<SettingsStore>().show_tray_badge();
            app.state::<TunnelState>().set_tray(tray, show_badge);
            shutdown::setup_signal_handler(app)?;
            shutdown::install_panic_cleanup(app);
            saved_tunnels::setup(app).expect("failed to initialize tunnel store");
            restart_saved_tunnels(app);

            Ok(())
        })
        .on_window_event(|window, event| {
            if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                api.prevent_close();
                let _ = window.hide();
            }
        })
        .build(tauri::generate_context!())
        .expect("error while building tauri application")
        .run(|app_handle, event| {
            // Belt-and-braces: make sure no orphaned ssh processes survive
            // the app exiting, however that exit is triggered. Note this
            // only kills the OS processes — it deliberately does NOT flip
            // each tunnel's persisted `running` flag, so they come back on
            // next launch via restart_saved_tunnels.
            if let tauri::RunEvent::ExitRequested { .. } = event {
                app_handle.state::<TunnelState>().stop_all();
            }
        });
}
