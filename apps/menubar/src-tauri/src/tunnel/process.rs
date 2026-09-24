use std::collections::VecDeque;
use std::io::{BufRead, BufReader};
use std::net::TcpStream;
use std::process::{Child, Command, Stdio};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

/// Our spawned ssh commands always carry both of these flags together —
/// used as a signature to identify our own processes among `ps` output,
/// for orphan detection.
pub const SIGNATURE_FLAGS: [&str; 2] = ["ExitOnForwardFailure=yes", "BatchMode=yes"];

pub const LOG_CAPACITY: usize = 200;
const HEALTH_PROBE_TIMEOUT: Duration = Duration::from_millis(400);

pub type LogBuffer = Arc<Mutex<VecDeque<String>>>;

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
pub fn probe_local_port(local_port: u16) -> Option<f64> {
    let addr = format!("127.0.0.1:{local_port}").parse().ok()?;
    let start = Instant::now();
    TcpStream::connect_timeout(&addr, HEALTH_PROBE_TIMEOUT)
        .ok()
        .map(|_| start.elapsed().as_secs_f64() * 1000.0)
}

/// The actual `ssh -L ...` spawn, shared between a fresh start and a
/// reconnect attempt.
pub fn spawn_ssh(
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

pub fn last_log_line(log: &LogBuffer) -> String {
    log.lock()
        .unwrap()
        .iter()
        .rev()
        .find(|line| !line.trim().is_empty())
        .cloned()
        .unwrap_or_else(|| "ssh exited unexpectedly".to_string())
}
