use std::process::Command;

/// PIDs of whatever is listening on `port` locally, via `lsof` — available
/// on both macOS and Linux.
pub fn port_owner_pids(port: u16) -> Vec<u32> {
    let output = match Command::new("lsof")
        .args(["-ti", &format!("tcp:{port}"), "-sTCP:LISTEN"])
        .output()
    {
        Ok(o) => o,
        Err(_) => return Vec::new(),
    };

    std::str::from_utf8(&output.stdout)
        .unwrap_or("")
        .split_whitespace()
        .filter_map(|s| s.parse().ok())
        .collect()
}

/// Every running process as (pid, full command line), via `ps`.
pub fn list_processes() -> Vec<(u32, String)> {
    let output = match Command::new("ps").args(["-eo", "pid=,command="]).output() {
        Ok(o) => o,
        Err(_) => return Vec::new(),
    };

    String::from_utf8_lossy(&output.stdout)
        .lines()
        .filter_map(|line| {
            let line = line.trim_start();
            let (pid_str, command) = line.split_once(char::is_whitespace)?;
            Some((pid_str.parse().ok()?, command.trim().to_string()))
        })
        .collect()
}

pub fn kill_pid(pid: u32) -> Result<(), String> {
    let status = Command::new("kill")
        .args(["-9", &pid.to_string()])
        .status()
        .map_err(|e| format!("failed to kill pid {pid}: {e}"))?;
    if !status.success() {
        return Err(format!("kill -9 {pid} failed"));
    }
    Ok(())
}
