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

/// Backoff schedule for auto-reconnect, indexed by attempt number (capped
/// at the last entry). After `MAX_RETRY_ATTEMPTS` consecutive failures we
/// give up rather than retry a permanently dead host forever.
const RETRY_DELAYS_SECS: [u64; 5] = [2, 4, 8, 16, 30];
const MAX_RETRY_ATTEMPTS: u32 = 6;

fn retry_delay(attempt: u32) -> Duration {
    let idx = (attempt as usize).min(RETRY_DELAYS_SECS.len() - 1);
    Duration::from_secs(RETRY_DELAYS_SECS[idx])
}

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
    /// Set only while `status` is `Retrying`.
    pub retry_attempt: Option<u32>,
    pub retry_in_secs: Option<u64>,
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

/// A tunnel is either actually forwarding right now, or between attempts
/// after dying, waiting for its backoff timer.
enum Runtime {
    Running { child: Child, pid: u32 },
    Retrying { attempt: u32, next_attempt: Instant, last_error: String },
}

struct TunnelHandle {
    ssh_host: String,
    local_port: u16,
    remote_host: String,
    remote_port: u16,
    runtime: Runtime,
    /// Persists across respawns — reconnect attempts keep appending to the
    /// same history instead of each attempt starting a fresh empty log.
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
///
/// Sub-millisecond precision matters here: this is a loopback connect, so
/// it routinely completes in well under 1ms — `.as_millis() as u64` would
/// truncate every real reading down to 0.
fn probe_local_port(local_port: u16) -> Option<f64> {
    let addr = format!("127.0.0.1:{local_port}").parse().ok()?;
    let start = Instant::now();
    TcpStream::connect_timeout(&addr, HEALTH_PROBE_TIMEOUT)
        .ok()
        .map(|_| start.elapsed().as_secs_f64() * 1000.0)
}

/// The actual `ssh -L ...` spawn, shared between a fresh start and a
/// reconnect attempt.
fn spawn_ssh(
    ssh_host: &str,
    local_port: u16,
    remote_host: &str,
    remote_port: u16,
    log: &LogBuffer,
) -> Result<(Child, u32), String> {
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
            ssh_host,
        ])
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| format!("failed to launch ssh: {e}"))?;

    if let Some(stderr) = child.stderr.take() {
        spawn_log_reader(stderr, log.clone());
    }

    let pid = child.id();
    Ok((child, pid))
}

fn last_log_line(log: &LogBuffer) -> String {
    log.lock()
        .unwrap()
        .iter()
        .rev()
        .find(|line| !line.trim().is_empty())
        .cloned()
        .unwrap_or_else(|| "ssh exited unexpectedly".to_string())
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

        let log: LogBuffer = Arc::new(Mutex::new(VecDeque::with_capacity(LOG_CAPACITY)));
        let (child, pid) = spawn_ssh(&ssh_host, local_port, &remote_host, remote_port, &log)?;

        self.tunnels.lock().unwrap().insert(
            id.clone(),
            TunnelHandle {
                ssh_host: ssh_host.clone(),
                local_port,
                remote_host: remote_host.clone(),
                remote_port,
                runtime: Runtime::Running { child, pid },
                log,
            },
        );

        Ok(TunnelInfo {
            id,
            ssh_host,
            local_port,
            remote_host,
            remote_port,
            pid: Some(pid),
            status: TunnelStatus::Connecting,
            latency_ms: None,
            retry_attempt: None,
            retry_in_secs: None,
        })
    }

    /// Manual stop: always fully removes tracking, killing the process if
    /// one is currently running, or just cancelling a pending retry.
    pub fn stop(&self, id: &str) -> Result<(), String> {
        let mut tunnels = self.tunnels.lock().unwrap();
        let Some(mut handle) = tunnels.remove(id) else {
            return Err(format!("no tunnel with id {id}"));
        };
        if let Runtime::Running { child, .. } = &mut handle.runtime {
            child.kill().map_err(|e| format!("failed to stop tunnel: {e}"))?;
            let _ = child.wait();
        }
        Ok(())
    }

    /// Advances retry timers, reaps dead processes into either a retry
    /// attempt or a permanent failure, probes survivors' ports, and
    /// returns the current view of everything still tracked.
    pub fn list(&self) -> Vec<TunnelInfo> {
        let mut tunnels = self.tunnels.lock().unwrap();
        let now = Instant::now();

        // 1. Processes that exited on their own since the last poll become
        //    a first retry attempt instead of an immediate failure.
        let newly_dead: Vec<String> = tunnels
            .iter_mut()
            .filter_map(|(id, handle)| match &mut handle.runtime {
                Runtime::Running { child, .. } => match child.try_wait() {
                    Ok(Some(_)) => Some(id.clone()),
                    _ => None,
                },
                Runtime::Retrying { .. } => None,
            })
            .collect();

        for id in &newly_dead {
            if let Some(handle) = tunnels.get_mut(id) {
                let last_error = last_log_line(&handle.log);
                handle.runtime = Runtime::Retrying {
                    attempt: 0,
                    next_attempt: now + retry_delay(0),
                    last_error,
                };
            }
        }

        // 2. Anything due for a retry attempt gets one. Success moves it
        //    back to Running; failure either schedules the next attempt or,
        //    past MAX_RETRY_ATTEMPTS, gives up for good.
        let due: Vec<String> = tunnels
            .iter()
            .filter_map(|(id, handle)| match &handle.runtime {
                Runtime::Retrying { next_attempt, .. } if now >= *next_attempt => {
                    Some(id.clone())
                }
                _ => None,
            })
            .collect();

        let mut gave_up = Vec::new();
        for id in due {
            let Some(handle) = tunnels.get_mut(&id) else { continue };
            let Runtime::Retrying { attempt, last_error, .. } = &handle.runtime else {
                continue;
            };
            let attempt = *attempt;
            let last_error = last_error.clone();

            match spawn_ssh(
                &handle.ssh_host,
                handle.local_port,
                &handle.remote_host,
                handle.remote_port,
                &handle.log,
            ) {
                Ok((child, pid)) => {
                    handle.runtime = Runtime::Running { child, pid };
                }
                Err(spawn_err) => {
                    let next = attempt + 1;
                    if next >= MAX_RETRY_ATTEMPTS {
                        gave_up.push((id.clone(), last_error));
                    } else {
                        handle.runtime = Runtime::Retrying {
                            attempt: next,
                            next_attempt: now + retry_delay(next),
                            last_error: spawn_err,
                        };
                    }
                }
            }
        }

        if !gave_up.is_empty() {
            let mut failures = self.failures.lock().unwrap();
            for (id, message) in gave_up {
                if let Some(handle) = tunnels.remove(&id) {
                    let port_in_use =
                        message.to_ascii_lowercase().contains("address already in use");
                    failures.push(TunnelFailure {
                        id,
                        ssh_host: handle.ssh_host,
                        local_port: handle.local_port,
                        remote_host: handle.remote_host,
                        remote_port: handle.remote_port,
                        message: format!("gave up after {MAX_RETRY_ATTEMPTS} attempts: {message}"),
                        port_in_use,
                    });
                }
            }
        }

        // 3. Build the view. Running entries get a fresh port probe;
        //    Retrying entries report their countdown instead.
        tunnels
            .iter_mut()
            .map(|(id, handle)| match &handle.runtime {
                Runtime::Running { pid, .. } => {
                    let latency_ms = probe_local_port(handle.local_port);
                    TunnelInfo {
                        id: id.clone(),
                        ssh_host: handle.ssh_host.clone(),
                        local_port: handle.local_port,
                        remote_host: handle.remote_host.clone(),
                        remote_port: handle.remote_port,
                        pid: Some(*pid),
                        status: if latency_ms.is_some() {
                            TunnelStatus::Connected
                        } else {
                            TunnelStatus::Connecting
                        },
                        latency_ms,
                        retry_attempt: None,
                        retry_in_secs: None,
                    }
                }
                Runtime::Retrying { attempt, next_attempt, .. } => TunnelInfo {
                    id: id.clone(),
                    ssh_host: handle.ssh_host.clone(),
                    local_port: handle.local_port,
                    remote_host: handle.remote_host.clone(),
                    remote_port: handle.remote_port,
                    pid: None,
                    status: TunnelStatus::Retrying,
                    latency_ms: None,
                    retry_attempt: Some(*attempt + 1),
                    retry_in_secs: Some(next_attempt.saturating_duration_since(now).as_secs()),
                },
            })
            .collect()
    }

    pub fn take_failures(&self) -> Vec<TunnelFailure> {
        std::mem::take(&mut *self.failures.lock().unwrap())
    }

    /// Snapshot of the recent stderr lines — spans every reconnect attempt
    /// for this tunnel, not just the current one.
    pub fn log(&self, id: &str) -> Option<Vec<String>> {
        let tunnels = self.tunnels.lock().unwrap();
        let handle = tunnels.get(id)?;
        let lines = handle.log.lock().unwrap().iter().cloned().collect();
        Some(lines)
    }

    pub fn stop_all(&self) {
        let mut tunnels = self.tunnels.lock().unwrap();
        for (_, handle) in tunnels.drain() {
            if let Runtime::Running { mut child, .. } = handle.runtime {
                let _ = child.kill();
                let _ = child.wait();
            }
        }
    }

    fn tracked_pids(&self) -> HashSet<u32> {
        self.tunnels
            .lock()
            .unwrap()
            .values()
            .filter_map(|h| match &h.runtime {
                Runtime::Running { pid, .. } => Some(*pid),
                Runtime::Retrying { .. } => None,
            })
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
