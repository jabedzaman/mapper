use std::process::Command;

/// Runs a PowerShell one-liner and returns its stdout. PowerShell (not
/// `cmd`) is the one shell guaranteed present since Windows 10 that can
/// actually query per-process network and command-line info.
fn run_powershell(script: &str) -> Option<String> {
    let output = Command::new("powershell")
        .args(["-NoProfile", "-NonInteractive", "-Command", script])
        .output()
        .ok()?;
    Some(String::from_utf8_lossy(&output.stdout).into_owned())
}

/// PIDs of whatever is listening on `port` locally, via
/// `Get-NetTCPConnection`.
pub fn port_owner_pids(port: u16) -> Vec<u32> {
    let script = format!(
        "Get-NetTCPConnection -LocalPort {port} -State Listen -ErrorAction SilentlyContinue | \
         Select-Object -ExpandProperty OwningProcess"
    );
    let Some(out) = run_powershell(&script) else {
        return Vec::new();
    };
    out.lines().filter_map(|l| l.trim().parse().ok()).collect()
}

/// Every running process as (pid, full command line), via CIM — `tasklist`
/// alone doesn't expose command-line arguments, which is what's needed to
/// recognize our own orphaned `ssh` invocations by their flags.
pub fn list_processes() -> Vec<(u32, String)> {
    let script = "Get-CimInstance Win32_Process | ForEach-Object { \"$($_.ProcessId)`t$($_.CommandLine)\" }";
    let Some(out) = run_powershell(script) else {
        return Vec::new();
    };
    out.lines()
        .filter_map(|line| {
            let (pid_str, command) = line.split_once('\t')?;
            Some((pid_str.trim().parse().ok()?, command.trim().to_string()))
        })
        .collect()
}

pub fn kill_pid(pid: u32) -> Result<(), String> {
    let status = Command::new("taskkill")
        .args(["/PID", &pid.to_string(), "/F"])
        .status()
        .map_err(|e| format!("failed to kill pid {pid}: {e}"))?;
    if !status.success() {
        return Err(format!("taskkill /PID {pid} /F failed"));
    }
    Ok(())
}
