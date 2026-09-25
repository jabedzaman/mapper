use serde::Serialize;

#[derive(Serialize, Clone, Debug, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum TunnelStatus {
    /// Process is running but the local port isn't accepting connections yet.
    Connecting,
    /// Process is running and the local port answered a probe connection.
    Connected,
    /// The process died and we're waiting to try spawning it again.
    Retrying,
}

#[derive(Serialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct TunnelInfo {
    pub id: String,
    pub ssh_host: String,
    pub local_port: u16,
    pub remote_host: String,
    pub remote_port: u16,
    pub pid: Option<u32>,
    pub status: TunnelStatus,
    pub latency_ms: Option<f64>,
    /// Seconds since the current connection last became healthy; `None`
    /// while connecting, retrying, or stopped. Resets on each reconnect.
    pub connected_secs: Option<u64>,
    /// Set only while `status` is `Retrying`.
    pub retry_attempt: Option<u32>,
    pub retry_in_secs: Option<u64>,
    /// Cumulative bytes since this connection attempt started, sampled
    /// best-effort from the OS. `None` when not running or when this
    /// platform has no way to sample it.
    pub bytes_received: Option<u64>,
    pub bytes_sent: Option<u64>,
}

#[derive(Serialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct TunnelFailure {
    pub id: String,
    pub ssh_host: String,
    pub local_port: u16,
    pub remote_host: String,
    pub remote_port: u16,
    pub message: String,
    /// True when the failure looks like the local port was already bound by
    /// something else, so the UI can offer to free it and retry.
    pub port_in_use: bool,
}

#[derive(Serialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct OrphanedProcess {
    pub pid: u32,
    pub command: String,
}
