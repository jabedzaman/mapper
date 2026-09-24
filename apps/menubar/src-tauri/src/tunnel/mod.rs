//! Everything to do with actually running ssh forwards: the live runtime
//! state machine (`state`), the raw process/log plumbing it spawns
//! (`process`), its reconnect backoff schedule (`retry`), the plain data
//! types shipped to the frontend (`types`), and standalone port/process
//! inspection helpers that don't need a `TunnelState` at all (`orphan`).

mod orphan;
mod process;
mod retry;
mod state;
mod types;

pub use orphan::{get_port_owners, kill_orphaned_ssh, kill_process_on_port, list_orphaned_ssh};
pub use state::TunnelState;
pub use types::{OrphanedProcess, TunnelFailure, TunnelInfo, TunnelStatus};
