use std::collections::HashMap;
use std::process::Command;

use super::process::SIGNATURE_FLAGS;
use super::state::TunnelState;
use super::types::OrphanedProcess;

/// Finds whatever is listening on `port` locally (via `lsof`), for showing
/// the user what "kill process & retry" is about to kill before they
/// confirm it.
pub fn get_port_owners(port: u16) -> Vec<OrphanedProcess> {
    let output = match Command::new("lsof")
        .args(["-ti", &format!("tcp:{port}"), "-sTCP:LISTEN"])
        .output()
    {
        Ok(o) => o,
        Err(_) => return Vec::new(),
    };

    let pids: Vec<u32> = std::str::from_utf8(&output.stdout)
        .unwrap_or("")
        .split_whitespace()
        .filter_map(|s| s.parse().ok())
        .collect();
    if pids.is_empty() {
        return Vec::new();
    }

    let commands: HashMap<u32, String> = match Command::new("ps").args(["-eo", "pid=,command="]).output() {
        Ok(o) => String::from_utf8_lossy(&o.stdout)
            .lines()
            .filter_map(|line| {
                let line = line.trim_start();
                let (pid_str, command) = line.split_once(char::is_whitespace)?;
                Some((pid_str.parse().ok()?, command.trim().to_string()))
            })
            .collect(),
        Err(_) => HashMap::new(),
    };

    pids.into_iter()
        .map(|pid| OrphanedProcess {
            pid,
            command: commands.get(&pid).cloned().unwrap_or_default(),
        })
        .collect()
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
