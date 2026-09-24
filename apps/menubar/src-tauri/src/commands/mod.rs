//! Tauri command handlers, grouped by the area of the app they belong to.
//! Each submodule owns its own imports; this file just re-exports
//! everything so callers keep using `commands::whatever` regardless of
//! which file it actually lives in.

mod app;
mod data;
mod processes;
mod ssh;
mod tunnels;

pub use app::*;
pub use data::*;
pub use processes::*;
pub use ssh::*;
pub use tunnels::*;
