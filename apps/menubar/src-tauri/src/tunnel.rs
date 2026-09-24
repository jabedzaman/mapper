use serde::Serialize;
use std::collections::{HashMap, HashSet, VecDeque};
use std::io::{BufRead, BufReader};
use std::net::TcpStream;
use std::process::{Child, Command, Stdio};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

/// Our spawned ssh commands always carry both of these flags together —
/// used as a signature to identify our own processes among `ps` output,
/// for orphan detection.
const SIGNATURE_FLAGS: [&str; 2] = ["ExitOnForwardFailure=yes", "BatchMode=yes"];

const LOG_CAPACITY: usize = 200;
const HEALTH_PROBE_TIMEOUT: Duration = Duration::from_millis(400);

#[derive(Serialize, Clone, Debug, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum TunnelStatus {
    /// Process is running but the local port isn't accepting connections yet.
    Connecting,
    /// Process is running and the local port answered a probe connection.
    Connected,
}

#[derive(Serialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct TunnelInfo {
    pub id: String,
    pub ssh_host: String,
    pub local_port: u16,
    pub remote_host: String,
    pub remote_port: u16,
    pub pid: u32,
    pub status: TunnelStatus,
    pub latency_ms: Option<u64>,
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

type LogBuffer = Arc<Mutex<VecDeque<String>>>;

struct TunnelHandle {
    child: Child,
    info: TunnelInfo,
    log: LogBuffer,
}

#[derive(Default)]
pub struct TunnelState {
    tunnels: Mutex<HashMap<String, TunnelHandle>>,
    failures: Mutex<Vec<TunnelFailure>>,
}

/// Reads stderr line-by-line for as long as the process lives, keeping only
/// the last `LOG_CAPACITY` lines. Runs on its own thread since pipe reads
/// block, and doesn't need the tunnels map lock at all — it only touches
/// its own buffer.
fn spawn_log_reader(stderr: std::process::ChildStderr, log: LogBuffer) {
    std::thread::spawn(move || {
        let reader = BufReader::new(stderr);
        for line in reader.lines() {
            let Ok(line) = line else { break };
            let mut buf = log.lock().unwrap();
            if buf.len() >= LOG_CAPACITY {
                buf.pop_front();
            }
            buf.push_back(line);
        }
    });
}

/// Attempts a short TCP connect to the forwarded local port. `Some(ms)` on
/// success (the forward is actually accepting connections), `None` if it
/// isn't up yet or the probe times out.
fn probe_local_port(local_port: u16) -> Option<u64> {
    let addr = format!("127.0.0.1:{local_port}").parse().ok()?;
    let start = Instant::now();
    TcpStream::connect_timeout(&addr, HEALTH_PROBE_TIMEOUT)
        .ok()
        .map(|_| start.elapsed().as_millis() as u64)
}

impl TunnelState {
    /// `id` is supplied by the caller (the persisted saved-tunnel id) so
    /// the runtime handle and the on-disk record always share one identity.
    pub fn start(
        &self,
        id: String,
        ssh_host: String,
        local_port: u16,
        remote_host: String,
        remote_port: u16,
    ) -> Result<TunnelInfo, String> {
        if self.tunnels.lock().unwrap().contains_key(&id) {
            return Err(format!("tunnel {id} is already running"));
        }

        let forward_spec = format!("{local_port}:{remote_host}:{remote_port}");

        let mut child = Command::new("ssh")
            .args([
                "-N", // no remote command, just forward
                "-o",
                SIGNATURE_FLAGS[0],
                "-o",
                SIGNATURE_FLAGS[1], // never block on an interactive password prompt
                "-o",
                "ServerAliveInterval=30",
                "-o",
                "ServerAliveCountMax=3",
                "-L",
                &forward_spec,
                &ssh_host,
            ])
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| format!("failed to launch ssh: {e}"))?;

        let log: LogBuffer = Arc::new(Mutex::new(VecDeque::with_capacity(LOG_CAPACITY)));
        if let Some(stderr) = child.stderr.take() {
            spawn_log_reader(stderr, log.clone());
        }

        let info = TunnelInfo {
            id: id.clone(),
            ssh_host,
            local_port,
            remote_host,
            remote_port,
            pid: child.id(),
            status: TunnelStatus::Connecting,
            latency_ms: None,
        };

        self.tunnels.lock().unwrap().insert(
            id,
            TunnelHandle {
                child,
                info: info.clone(),
                log,
            },
        );

        Ok(info)
    }

    pub fn stop(&self, id: &str) -> Result<(), String> {
        let mut tunnels = self.tunnels.lock().unwrap();
        let Some(mut handle) = tunnels.remove(id) else {
            return Err(format!("no tunnel with id {id}"));
        };
        handle
            .child
            .kill()
            .map_err(|e| format!("failed to stop tunnel: {e}"))?;
        let _ = handle.child.wait();
        Ok(())
    }

    /// Prunes ssh processes that exited on their own, probes the survivors'
    /// local ports to refresh status/latency, and returns the live list.
    pub fn list(&self) -> Vec<TunnelInfo> {
        let mut tunnels = self.tunnels.lock().unwrap();

        let dead: Vec<String> = tunnels
            .iter_mut()
            .filter_map(|(id, handle)| match handle.child.try_wait() {
                Ok(Some(_)) => Some(id.clone()),
                _ => None,
            })
            .collect();

        if !dead.is_empty() {
            let mut failures = self.failures.lock().unwrap();
            for id in dead {
                if let Some(handle) = tunnels.remove(&id) {
                    let log = handle.log.lock().unwrap();
                    let message = log
                        .iter()
                        .rev()
                        .find(|line| !line.trim().is_empty())
                        .cloned()
                        .unwrap_or_else(|| "ssh exited unexpectedly".to_string());
                    let port_in_use =
                        message.to_ascii_lowercase().contains("address already in use");
                    failures.push(TunnelFailure {
                        id: handle.info.id.clone(),
                        ssh_host: handle.info.ssh_host.clone(),
                        local_port: handle.info.local_port,
                        remote_host: handle.info.remote_host.clone(),
                        remote_port: handle.info.remote_port,
                        message,
                        port_in_use,
                    });
                }
            }
        }

        tunnels
            .values_mut()
            .map(|handle| {
                let latency_ms = probe_local_port(handle.info.local_port);
                handle.info.status = if latency_ms.is_some() {
                    TunnelStatus::Connected
                } else {
                    TunnelStatus::Connecting
                };
                handle.info.latency_ms = latency_ms;
                handle.info.clone()
            })
            .collect()
    }

    pub fn take_failures(&self) -> Vec<TunnelFailure> {
        std::mem::take(&mut *self.failures.lock().unwrap())
    }

    /// Snapshot of the recent stderr lines for a running tunnel.
    pub fn log(&self, id: &str) -> Option<Vec<String>> {
        let tunnels = self.tunnels.lock().unwrap();
        let handle = tunnels.get(id)?;
        let lines = handle.log.lock().unwrap().iter().cloned().collect();
        Some(lines)
    }

    pub fn stop_all(&self) {
        let mut tunnels = self.tunnels.lock().unwrap();
        for (_, mut handle) in tunnels.drain() {
            let _ = handle.child.kill();
            let _ = handle.child.wait();
        }
    }

    fn tracked_pids(&self) -> HashSet<u32> {
        self.tunnels
            .lock()
            .unwrap()
            .values()
            .map(|h| h.info.pid)
            .collect()
    }
}

/// Finds whatever is listening on `port` locally (via `lsof`) and kills it.
/// Used to recover from "Address already in use" when starting a forward.
pub fn kill_process_on_port(port: u16) -> Result<(), String> {
    let output = Command::new("lsof")
        .args(["-ti", &format!("tcp:{port}"), "-sTCP:LISTEN"])
        .output()
        .map_err(|e| format!("failed to run lsof: {e}"))?;

    let pids: Vec<&str> = std::str::from_utf8(&output.stdout)
        .unwrap_or("")
        .split_whitespace()
        .collect();

    if pids.is_empty() {
        return Err(format!("no process found listening on port {port}"));
    }

    for pid in pids {
        let status = Command::new("kill")
            .args(["-9", pid])
            .status()
            .map_err(|e| format!("failed to kill pid {pid}: {e}"))?;
        if !status.success() {
            return Err(format!("kill -9 {pid} failed"));
        }
    }

    Ok(())
}

/// Finds `ssh` processes carrying our forward signature that this
/// `TunnelState` isn't currently tracking — i.e. leftovers from a previous
/// crashed run of the app.
pub fn list_orphaned_ssh(state: &TunnelState) -> Vec<OrphanedProcess> {
    let tracked = state.tracked_pids();

    let output = match Command::new("ps").args(["-eo", "pid=,command="]).output() {
        Ok(o) => o,
        Err(_) => return Vec::new(),
    };
    let text = String::from_utf8_lossy(&output.stdout);

    text.lines()
        .filter_map(|line| {
            let line = line.trim_start();
            let (pid_str, command) = line.split_once(char::is_whitespace)?;
            let pid: u32 = pid_str.parse().ok()?;
            if tracked.contains(&pid) {
                return None;
            }
            let command = command.trim();
            let looks_like_ours = command.starts_with("ssh ")
                && SIGNATURE_FLAGS.iter().all(|flag| command.contains(flag));
            looks_like_ours.then(|| OrphanedProcess {
                pid,
                command: command.to_string(),
            })
        })
        .collect()
}

pub fn kill_orphaned_ssh(pid: u32) -> Result<(), String> {
    let status = Command::new("kill")
        .args(["-9", &pid.to_string()])
        .status()
        .map_err(|e| format!("failed to kill pid {pid}: {e}"))?;
    if !status.success() {
        return Err(format!("kill -9 {pid} failed"));
    }
    Ok(())
}
