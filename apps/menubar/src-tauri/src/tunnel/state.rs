use std::collections::{HashMap, HashSet, VecDeque};
use std::process::Child;
use std::sync::{Arc, Mutex};
use std::time::Instant;

use tauri::tray::TrayIcon;

use super::process::{last_log_line, spawn_ssh, LogBuffer, LOG_CAPACITY};
use super::retry::{retry_delay, MAX_RETRY_ATTEMPTS};
use super::types::{TunnelFailure, TunnelInfo, TunnelStatus};

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
    /// When the current connection last became healthy (the local port
    /// answered a probe). Reset on drop, giving per-connection uptime.
    connected_at: Option<Instant>,
}

#[derive(Default)]
pub struct TunnelState {
    tunnels: Mutex<HashMap<String, TunnelHandle>>,
    failures: Mutex<Vec<TunnelFailure>>,
    /// Handle to the menu-bar tray icon, set during app setup. Used to show
    /// the count of tunnels with a live forward process as tray title text.
    tray: Mutex<Option<TrayIcon>>,
    /// Whether the tray badge is enabled (user toggle). Defaults to on.
    show_badge: Mutex<bool>,
}

impl TunnelState {
    /// Hands the state the live tray icon so it can badge the menu-bar
    /// item with the active tunnel count. Called once during app setup.
    pub fn set_tray(&self, tray: TrayIcon, show_badge: bool) {
        self.tray.lock().unwrap().replace(tray);
        *self.show_badge.lock().unwrap() = show_badge;
        self.update_badge();
    }

    /// Flips whether the tray badge is shown. Persisted separately in the
    /// settings store; this only mirrors it onto runtime behavior.
    pub fn set_badge_visible(&self, visible: bool) {
        *self.show_badge.lock().unwrap() = visible;
        self.update_badge();
    }

    /// Handle to the tray icon, for rebuilding its dropdown menu. `None`
    /// only before `set_tray` runs during app setup.
    pub fn tray(&self) -> Option<TrayIcon> {
        self.tray.lock().unwrap().clone()
    }

    /// Number of tunnels with a live ssh process right now. Retrying
    /// tunnels don't count — their forward is actually down.
    fn running_count(&self) -> usize {
        self.tunnels
            .lock()
            .unwrap()
            .values()
            .filter(|h| matches!(h.runtime, Runtime::Running { .. }))
            .count()
    }

    /// Pushes the active tunnel count into the tray title; clears it when
    /// nothing is running so the plain icon stays uncluttered. When the
    /// badge is disabled, keeps the title cleared entirely. Best-effort:
    /// a failure here never affects tunnel operations.
    ///
    /// Note: `set_title(None)` is a no-op on macOS (tray-icon only applies
    /// `Some`), so clearing must use an empty string.
    const EMPTY_TITLE: Option<&str> = Some("");

    fn update_badge(&self) {
        if !*self.show_badge.lock().unwrap() {
            if let Some(tray) = self.tray.lock().unwrap().as_ref() {
                let _ = tray.set_title(Self::EMPTY_TITLE);
            }
            return;
        }

        let n = self.running_count();
        if let Some(tray) = self.tray.lock().unwrap().as_ref() {
            if n == 0 {
                let _ = tray.set_title(Self::EMPTY_TITLE);
            } else {
                let _ = tray.set_title(Some(n.to_string()));
            }
        }
    }

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
                connected_at: None,
            },
        );
        self.update_badge();

        Ok(TunnelInfo {
            id,
            ssh_host,
            local_port,
            remote_host,
            remote_port,
            pid: Some(pid),
            status: TunnelStatus::Connecting,
            latency_ms: None,
            connected_secs: None,
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
        self.update_badge();
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
                handle.connected_at = None;
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
        let view: Vec<TunnelInfo> = tunnels
            .iter_mut()
            .map(|(id, handle)| match &handle.runtime {
                Runtime::Running { pid, .. } => {
                    let latency_ms = super::process::probe_local_port(handle.local_port);
                    let connected = latency_ms.is_some();
                    if connected && handle.connected_at.is_none() {
                        handle.connected_at = Some(now);
                    } else if !connected {
                        handle.connected_at = None;
                    }
                    let connected_secs = connected
                        .then(|| handle.connected_at.map(|at| now.duration_since(at).as_secs()))
                        .flatten();
                    TunnelInfo {
                        id: id.clone(),
                        ssh_host: handle.ssh_host.clone(),
                        local_port: handle.local_port,
                        remote_host: handle.remote_host.clone(),
                        remote_port: handle.remote_port,
                        pid: Some(*pid),
                        status: if connected {
                            TunnelStatus::Connected
                        } else {
                            TunnelStatus::Connecting
                        },
                        latency_ms,
                        connected_secs,
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
                    connected_secs: None,
                    retry_attempt: Some(*attempt + 1),
                    retry_in_secs: Some(next_attempt.saturating_duration_since(now).as_secs()),
                },
            })
            .collect();
        drop(tunnels);
        self.update_badge();
        view
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
        drop(tunnels);
        self.update_badge();
    }

    /// Exposed to `super::orphan` so it can tell apart our own tracked ssh
    /// processes from genuine leftovers of a previous crashed run.
    pub(super) fn tracked_pids(&self) -> HashSet<u32> {
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
