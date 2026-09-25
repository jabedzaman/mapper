//! The handful of OS-shelling-out primitives that differ between
//! platforms: who's listening on a port, what's currently running, and
//! how to kill a pid. Everything that builds on top of these (matching
//! our own ssh processes by signature, merging port owners with process
//! names, ...) is platform-agnostic and lives in `crate::tunnel::orphan`
//! instead of being duplicated per OS here.
//!
//! macOS and Linux share an implementation (`lsof`/`ps`/`kill` are common
//! to both); Windows gets its own via PowerShell.

mod bandwidth;
#[cfg(unix)]
mod unix;
#[cfg(windows)]
mod windows;

pub use bandwidth::sample_bytes;
#[cfg(unix)]
pub use unix::{kill_pid, list_processes, port_owner_pids};
#[cfg(windows)]
pub use windows::{kill_pid, list_processes, port_owner_pids};
