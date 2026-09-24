mod commands;
mod shutdown;
mod ssh_config;
mod tray;
mod tunnel;

use tauri::Manager;
use tunnel::TunnelState;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .manage(TunnelState::default())
        .invoke_handler(tauri::generate_handler![
            commands::list_ssh_hosts,
            commands::list_tunnels,
            commands::start_tunnel,
            commands::stop_tunnel,
            commands::take_tunnel_failures,
            commands::kill_process_on_port,
            commands::quit_app,
        ])
        .setup(|app| {
            // Menu-bar-only app: no Dock icon, no app menu bar.
            #[cfg(target_os = "macos")]
            app.set_activation_policy(tauri::ActivationPolicy::Accessory);

            tray::setup(app)?;
            shutdown::setup_signal_handler(app)?;
            shutdown::install_panic_cleanup(app);

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
            // the app exiting, however that exit is triggered.
            if let tauri::RunEvent::ExitRequested { .. } = event {
                app_handle.state::<TunnelState>().stop_all();
            }
        });
}
