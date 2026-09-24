//! The on-disk record of every tunnel the user has ever started: plain
//! data (`types`) plus the JSON-file-backed store built around it
//! (`store`). Runtime process state lives entirely in `crate::tunnel`
//! instead — this module only tracks what should exist and whether it's
//! meant to be running.

mod store;
mod types;

pub use store::SavedTunnelStore;

use tauri::{App, Manager};

pub fn setup(app: &App) -> Result<(), String> {
    let store = SavedTunnelStore::load(&app.handle())?;
    app.manage(store);
    Ok(())
}
