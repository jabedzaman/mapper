use serde::Serialize;
use std::collections::HashMap;
use std::io::Read;
use std::process::{Child, Command, Stdio};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Mutex;

#[derive(Serialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct TunnelInfo {
    pub id: String,
    pub ssh_host: String,
    pub local_port: u16,
    pub remote_host: String,
    pub remote_port: u16,
    pub pid: u32,
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

struct TunnelHandle {
    child: Child,
    info: TunnelInfo,
}

#[derive(Default)]
pub struct TunnelState {
    tunnels: Mutex<HashMap<String, TunnelHandle>>,
    failures: Mutex<Vec<TunnelFailure>>,
}

static NEXT_ID: AtomicU64 = AtomicU64::new(1);

impl TunnelState {
    pub fn start(
        &self,
        ssh_host: String,
        local_port: u16,
        remote_host: String,
        remote_port: u16,
    ) -> Result<TunnelInfo, String> {
        let forward_spec = format!("{local_port}:{remote_host}:{remote_port}");

        let child = Command::new("ssh")
            .args([
                "-N", // no remote command, just forward
                "-o",
                "ExitOnForwardFailure=yes",
                "-o",
                "BatchMode=yes", // never block on an interactive password prompt
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

        let id = NEXT_ID.fetch_add(1, Ordering::SeqCst).to_string();
        let info = TunnelInfo {
            id: id.clone(),
            ssh_host,
            local_port,
            remote_host,
            remote_port,
            pid: child.id(),
        };

        self.tunnels
            .lock()
            .unwrap()
            .insert(id, TunnelHandle { child, info: info.clone() });

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

    pub fn list(&self) -> Vec<TunnelInfo> {
        let mut tunnels = self.tunnels.lock().unwrap();
        // Prune any ssh processes that exited on their own (auth failure,
        // network drop, host went away) so the UI reflects reality, and
        // capture why so the frontend can show it instead of the tunnel
        // just silently vanishing from the list.
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
                if let Some(mut handle) = tunnels.remove(&id) {
                    let mut message = String::new();
                    if let Some(mut stderr) = handle.child.stderr.take() {
                        let _ = stderr.read_to_string(&mut message);
                    }
                    let message = message.trim();
                    let message = if message.is_empty() {
                        "ssh exited unexpectedly".to_string()
                    } else {
                        message.to_string()
                    };
                    let port_in_use = message.to_ascii_lowercase().contains("address already in use");
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

        tunnels.values().map(|h| h.info.clone()).collect()
    }

    pub fn take_failures(&self) -> Vec<TunnelFailure> {
        std::mem::take(&mut *self.failures.lock().unwrap())
    }

    pub fn stop_all(&self) {
        let mut tunnels = self.tunnels.lock().unwrap();
        for (_, mut handle) in tunnels.drain() {
            let _ = handle.child.kill();
            let _ = handle.child.wait();
        }
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
