use std::collections::HashMap;

use crate::platform;

use super::process::SIGNATURE_FLAGS;
use super::state::TunnelState;
use super::types::OrphanedProcess;

/// Finds whatever is listening on `port` locally, for showing the user
/// what "kill process & retry" is about to kill before they confirm it.
pub fn get_port_owners(port: u16) -> Vec<OrphanedProcess> {
    let pids = platform::port_owner_pids(port);
    if pids.is_empty() {
        return Vec::new();
    }

    let commands: HashMap<u32, String> = platform::list_processes().into_iter().collect();

    pids.into_iter()
        .map(|pid| OrphanedProcess {
            pid,
            command: commands.get(&pid).cloned().unwrap_or_default(),
        })
        .collect()
}

/// Finds whatever is listening on `port` locally and kills it. Used to
/// recover from "Address already in use" when starting a forward.
pub fn kill_process_on_port(port: u16) -> Result<(), String> {
    let pids = platform::port_owner_pids(port);
    if pids.is_empty() {
        return Err(format!("no process found listening on port {port}"));
    }
    for pid in pids {
        platform::kill_pid(pid)?;
    }
    Ok(())
}

/// Finds `ssh` processes carrying our forward signature that this
/// `TunnelState` isn't currently tracking — i.e. leftovers from a previous
/// crashed run of the app.
pub fn list_orphaned_ssh(state: &TunnelState) -> Vec<OrphanedProcess> {
    let tracked = state.tracked_pids();

    platform::list_processes()
        .into_iter()
        .filter(|(pid, _)| !tracked.contains(pid))
        .filter_map(|(pid, command)| {
            let lower = command.to_ascii_lowercase();
            let looks_like_ours =
                lower.contains("ssh") && SIGNATURE_FLAGS.iter().all(|flag| command.contains(flag));
            looks_like_ours.then(|| OrphanedProcess { pid, command })
        })
        .collect()
}

pub fn kill_orphaned_ssh(pid: u32) -> Result<(), String> {
    platform::kill_pid(pid)
}
