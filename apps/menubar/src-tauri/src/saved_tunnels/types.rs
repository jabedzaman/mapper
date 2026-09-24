use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};

/// Timestamp-based rather than an incrementing counter — a counter resets
/// to 1 on every process start, which would collide with ids already on
/// disk from a previous run.
pub(super) fn generate_id() -> String {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    nanos.to_string()
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct SavedTunnel {
    pub id: String,
    pub name: String,
    pub ssh_host: String,
    pub local_port: u16,
    pub remote_host: String,
    pub remote_port: u16,
    /// The user's intent, not live process state: should this forward be
    /// running? Restored on launch (auto-starts it), and cleared whenever
    /// the process is stopped — deliberately or because it died.
    #[serde(default)]
    pub running: bool,
}
