/// Best-effort cumulative (bytes_received, bytes_sent) for `pid` since it
/// started, or `None` when sampling isn't supported or failed. This is
/// best-effort by nature — none of these are a guaranteed OS API, just
/// the closest each platform gets to a per-process byte counter without
/// elevated privileges or packet-capture-level integration.
///
/// Every implementation runs its subprocess through `run_with_timeout`
/// rather than a plain `.output()`: a hung sampling command must never be
/// able to block the caller indefinitely. This isn't hypothetical — a
/// GUI-launched `.app` bundle can hit macOS permission prompts that never
/// surface properly for a background/accessory app, and a caller that
/// blocks forever here previously froze the whole app, since this runs
/// while the caller holds the tunnel state lock.
#[cfg(any(target_os = "macos", target_os = "linux"))]
fn run_with_timeout(mut cmd: std::process::Command, timeout: std::time::Duration) -> Option<String> {
    use std::io::Read;
    use wait_timeout::ChildExt;

    let mut child = cmd
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::null())
        .spawn()
        .ok()?;

    match child.wait_timeout(timeout) {
        Ok(Some(_status)) => {
            let mut out = String::new();
            child.stdout.take()?.read_to_string(&mut out).ok()?;
            Some(out)
        }
        _ => {
            // Either it timed out (Ok(None)) or the wait itself failed —
            // either way, don't leave a zombie or a runaway process behind.
            let _ = child.kill();
            let _ = child.wait();
            None
        }
    }
}

#[cfg(any(target_os = "macos", target_os = "linux"))]
const SAMPLE_TIMEOUT: std::time::Duration = std::time::Duration::from_millis(800);

/// Via `nettop`'s per-process aggregate row — the unindented line, as
/// opposed to the indented per-connection breakdown beneath it (a
/// forwarded port can have more than one open connection under the same
/// ssh pid, and `nettop` already sums those for us there).
///
/// `-n` (skip reverse-DNS on the peer address) is not optional here: without
/// it, each call spends ~5s blocked on address resolution — measured, not
/// assumed — which is what made the whole app feel like it was lagging once
/// this ran every poll tick. With it, a call takes ~15ms.
#[cfg(target_os = "macos")]
pub fn sample_bytes(pid: u32) -> Option<(u64, u64)> {
    let mut cmd = std::process::Command::new("nettop");
    cmd.args(["-p", &pid.to_string(), "-x", "-n", "-l", "1", "-J", "bytes_in,bytes_out"]);
    let text = run_with_timeout(cmd, SAMPLE_TIMEOUT)?;

    for line in text.lines() {
        if line.is_empty() || line.starts_with(char::is_whitespace) {
            continue; // the header row, or a per-connection sub-row
        }
        let mut nums = line.split_whitespace().rev();
        let bytes_out: u64 = nums.next()?.parse().ok()?;
        let bytes_in: u64 = nums.next()?.parse().ok()?;
        return Some((bytes_in, bytes_out));
    }
    None
}

/// Via `ss`'s extended tcp_info line, summed across every socket this pid
/// owns. `bytes_sent` is only reported by newer kernels (~4.6+); older
/// ones only expose `bytes_acked`, which is close enough for a running
/// total.
#[cfg(target_os = "linux")]
pub fn sample_bytes(pid: u32) -> Option<(u64, u64)> {
    let mut cmd = std::process::Command::new("ss");
    cmd.args(["-tinp"]);
    let text = run_with_timeout(cmd, SAMPLE_TIMEOUT)?;

    let lines: Vec<&str> = text.lines().collect();
    let needle = format!("pid={pid},");

    let mut total_in = 0u64;
    let mut total_out = 0u64;
    let mut found = false;

    for (i, line) in lines.iter().enumerate() {
        if !line.contains(&needle) {
            continue;
        }
        let Some(info) = lines.get(i + 1) else { continue };
        found = true;
        if let Some(v) = extract_metric(info, "bytes_received:") {
            total_in += v;
        }
        if let Some(v) =
            extract_metric(info, "bytes_sent:").or_else(|| extract_metric(info, "bytes_acked:"))
        {
            total_out += v;
        }
    }

    found.then_some((total_in, total_out))
}

#[cfg(target_os = "linux")]
fn extract_metric(line: &str, key: &str) -> Option<u64> {
    line.split(key).nth(1)?.split_whitespace().next()?.parse().ok()
}

/// No built-in per-process byte counter on Windows without an ETW/perfmon
/// integration — left unsupported rather than faking a number.
#[cfg(target_os = "windows")]
pub fn sample_bytes(_pid: u32) -> Option<(u64, u64)> {
    None
}
