use signal_hook::consts::{SIGINT, SIGTERM};
use signal_hook::iterator::Signals;
use tauri::{App, Manager};

use crate::tunnel::TunnelState;

/// Quit via the app's own Quit button and normal window-manager exits are
/// already handled by Tauri's event loop. This covers the paths that
/// bypass it entirely — `kill <pid>`, logout, Activity Monitor "Quit" —
/// which would otherwise orphan the `ssh` child processes.
///
/// SIGKILL can't be caught by any process — a `kill -9` is the one
/// external-termination path this can't clean up after, and that's an OS
/// guarantee, not a gap to work around.
pub fn setup_signal_handler(app: &App) -> std::io::Result<()> {
    let mut signals = Signals::new([SIGTERM, SIGINT])?;
    let handle = app.handle().clone();

    std::thread::spawn(move || {
        if signals.forever().next().is_some() {
            handle.state::<TunnelState>().stop_all();
            handle.exit(0);
        }
    });

    Ok(())
}

/// Covers the other kind of "crash": a Rust panic in this process (a
/// stray `.unwrap()`, an index out of bounds, a poisoned mutex, ...).
/// The release profile aborts on panic, but the panic hook still runs
/// first, so this gets one chance to kill tracked ssh children before
/// that happens.
///
/// This is best-effort by nature, and it stops at the language boundary:
/// undefined behavior from unsafe code (segfaults, etc.) doesn't go
/// through Rust's panic machinery at all, so there's nothing here to
/// catch it — this app doesn't use unsafe, so that risk is already
/// close to zero, not something patched over by a signal handler.
pub fn install_panic_cleanup(app: &App) {
    let handle = app.handle().clone();
    let default_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        handle.state::<TunnelState>().stop_all();
        default_hook(info);
    }));
}
