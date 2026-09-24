use std::collections::HashSet;
use std::fs;
use std::path::PathBuf;
use std::sync::Mutex;

use tauri::{AppHandle, Manager};

use super::types::{generate_id, SavedTunnel};

pub struct SavedTunnelStore {
    path: PathBuf,
    tunnels: Mutex<Vec<SavedTunnel>>,
}

impl SavedTunnelStore {
    pub fn load(app: &AppHandle) -> Result<Self, String> {
        let dir = app
            .path()
            .app_config_dir()
            .map_err(|e| format!("failed to resolve config dir: {e}"))?;
        fs::create_dir_all(&dir).map_err(|e| format!("failed to create config dir: {e}"))?;
        let path = dir.join("tunnels.json");

        let tunnels = match fs::read_to_string(&path) {
            Ok(contents) => serde_json::from_str(&contents).unwrap_or_default(),
            Err(_) => Vec::new(),
        };

        Ok(Self {
            path,
            tunnels: Mutex::new(tunnels),
        })
    }

    fn persist(&self, tunnels: &[SavedTunnel]) -> Result<(), String> {
        let json = serde_json::to_string_pretty(tunnels)
            .map_err(|e| format!("failed to serialize tunnels: {e}"))?;
        fs::write(&self.path, json).map_err(|e| format!("failed to write tunnels file: {e}"))
    }

    pub fn list(&self) -> Vec<SavedTunnel> {
        self.tunnels.lock().unwrap().clone()
    }

    /// Finds an existing saved tunnel matching this exact connection, or
    /// creates one. Either way marks it running and persists — this is
    /// the entry point for both "start a brand new forward" and "start a
    /// previously stopped one via the same host/ports".
    pub fn start(
        &self,
        ssh_host: &str,
        local_port: u16,
        remote_host: &str,
        remote_port: u16,
    ) -> SavedTunnel {
        let mut tunnels = self.tunnels.lock().unwrap();

        if let Some(t) = tunnels.iter_mut().find(|t| {
            t.ssh_host == ssh_host
                && t.local_port == local_port
                && t.remote_host == remote_host
                && t.remote_port == remote_port
        }) {
            t.running = true;
            let result = t.clone();
            let _ = self.persist(&tunnels);
            return result;
        }

        let t = SavedTunnel {
            id: generate_id(),
            name: format!("{ssh_host} · {local_port} → {remote_host}:{remote_port}"),
            ssh_host: ssh_host.to_string(),
            local_port,
            remote_host: remote_host.to_string(),
            remote_port,
            running: true,
        };
        tunnels.push(t.clone());
        let _ = self.persist(&tunnels);
        t
    }

    /// Starts a specific already-saved tunnel by id (used for "Start" on a
    /// stopped list entry — same host/ports, no risk of creating a dupe).
    pub fn get(&self, id: &str) -> Option<SavedTunnel> {
        self.tunnels.lock().unwrap().iter().find(|t| t.id == id).cloned()
    }

    pub fn set_running(&self, id: &str, running: bool) -> Result<(), String> {
        let mut tunnels = self.tunnels.lock().unwrap();
        let Some(t) = tunnels.iter_mut().find(|t| t.id == id) else {
            return Err(format!("no tunnel with id {id}"));
        };
        t.running = running;
        self.persist(&tunnels)
    }

    /// Edits an existing saved tunnel's connection in place — same id, so
    /// callers (and any live runtime handle) don't need to know it changed
    /// identity. Caller is responsible for restarting the process if it
    /// was running.
    pub fn update(
        &self,
        id: &str,
        ssh_host: &str,
        local_port: u16,
        remote_host: &str,
        remote_port: u16,
    ) -> Result<SavedTunnel, String> {
        let mut tunnels = self.tunnels.lock().unwrap();
        let Some(t) = tunnels.iter_mut().find(|t| t.id == id) else {
            return Err(format!("no tunnel with id {id}"));
        };
        t.ssh_host = ssh_host.to_string();
        t.local_port = local_port;
        t.remote_host = remote_host.to_string();
        t.remote_port = remote_port;
        t.name = format!("{ssh_host} · {local_port} → {remote_host}:{remote_port}");
        let result = t.clone();
        self.persist(&tunnels)?;
        Ok(result)
    }

    pub fn remove(&self, id: &str) -> Result<(), String> {
        let mut tunnels = self.tunnels.lock().unwrap();
        let before = tunnels.len();
        tunnels.retain(|t| t.id != id);
        if tunnels.len() == before {
            return Err(format!("no tunnel with id {id}"));
        }
        self.persist(&tunnels)
    }

    /// Called each poll: any tunnel marked running whose process isn't
    /// actually alive gets flipped back to stopped — it either died on its
    /// own, or a start attempt never actually launched.
    pub fn reconcile(&self, live_ids: &HashSet<String>) {
        let mut tunnels = self.tunnels.lock().unwrap();
        let mut changed = false;
        for t in tunnels.iter_mut() {
            if t.running && !live_ids.contains(&t.id) {
                t.running = false;
                changed = true;
            }
        }
        if changed {
            let _ = self.persist(&tunnels);
        }
    }

    /// Writes the current saved tunnels out to an arbitrary file, for
    /// sharing. Running state isn't meaningful outside this machine, so it
    /// isn't included.
    pub fn export_to(&self, path: &str) -> Result<(), String> {
        let tunnels = self.tunnels.lock().unwrap();
        let json = serde_json::to_string_pretty(&*tunnels)
            .map_err(|e| format!("failed to serialize tunnels: {e}"))?;
        fs::write(path, json).map_err(|e| format!("failed to write {path}: {e}"))
    }

    /// Merges tunnels read from an arbitrary file into the store, deduped
    /// on the connection itself (same as `start`). Imported tunnels always
    /// come in stopped — never auto-connect to a host just because it was
    /// in a file someone handed you.
    pub fn import_from(&self, path: &str) -> Result<usize, String> {
        let contents = fs::read_to_string(path).map_err(|e| format!("failed to read {path}: {e}"))?;
        let imported: Vec<SavedTunnel> =
            serde_json::from_str(&contents).map_err(|e| format!("invalid tunnels file: {e}"))?;

        let mut tunnels = self.tunnels.lock().unwrap();
        let mut existing: HashSet<(String, u16, String, u16)> = tunnels
            .iter()
            .map(|t| (t.ssh_host.clone(), t.local_port, t.remote_host.clone(), t.remote_port))
            .collect();

        let mut added = 0;
        for mut tunnel in imported {
            let key = (
                tunnel.ssh_host.clone(),
                tunnel.local_port,
                tunnel.remote_host.clone(),
                tunnel.remote_port,
            );
            if !existing.insert(key) {
                continue;
            }
            tunnel.id = generate_id();
            tunnel.running = false;
            tunnels.push(tunnel);
            added += 1;
        }

        self.persist(&tunnels)?;
        Ok(added)
    }
}
