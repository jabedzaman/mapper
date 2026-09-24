use serde::Serialize;
use std::fs;
use std::net::{TcpStream, ToSocketAddrs};
use std::time::{Duration, Instant};

const CHECK_TIMEOUT: Duration = Duration::from_millis(1500);

#[derive(Serialize, Clone, Debug)]
pub struct SshHost {
    pub alias: String,
    pub hostname: Option<String>,
    pub user: Option<String>,
    pub port: Option<u16>,
}

#[derive(Serialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct HostCheck {
    pub reachable: bool,
    pub latency_ms: Option<f64>,
    pub error: Option<String>,
}

/// Parses `Host` blocks out of `~/.ssh/config`. Wildcard/pattern aliases
/// (containing `*` or `?`) are skipped since they aren't a real target
/// to forward through.
pub fn list_hosts() -> Vec<SshHost> {
    let Some(home) = dirs::home_dir() else {
        return Vec::new();
    };
    let path = home.join(".ssh").join("config");
    let Ok(contents) = fs::read_to_string(path) else {
        return Vec::new();
    };

    let mut hosts: Vec<SshHost> = Vec::new();
    let mut current: Option<SshHost> = None;

    for raw_line in contents.lines() {
        let line = raw_line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let mut parts = line.splitn(2, char::is_whitespace);
        let Some(key) = parts.next() else { continue };
        let value = parts.next().unwrap_or("").trim();
        let key_lower = key.to_ascii_lowercase();

        if key_lower == "host" {
            if let Some(host) = current.take() {
                hosts.push(host);
            }
            for alias in value.split_whitespace() {
                if alias.contains('*') || alias.contains('?') {
                    continue;
                }
                current = Some(SshHost {
                    alias: alias.to_string(),
                    hostname: None,
                    user: None,
                    port: None,
                });
                break;
            }
            continue;
        }

        let Some(host) = current.as_mut() else { continue };
        match key_lower.as_str() {
            "hostname" => host.hostname = Some(value.to_string()),
            "user" => host.user = Some(value.to_string()),
            "port" => host.port = value.parse().ok(),
            _ => {}
        }
    }
    if let Some(host) = current.take() {
        hosts.push(host);
    }

    hosts
}

/// Resolves an alias/host string to the (hostname, port) to actually dial:
/// prefers what `~/.ssh/config` says for that alias, then falls back to
/// treating the input as a literal `user@host` or bare host with the
/// default ssh port — covers the manual-entry mode too.
fn resolve_target(alias: &str) -> (String, u16) {
    if let Some(host) = list_hosts().into_iter().find(|h| h.alias == alias) {
        return (host.hostname.unwrap_or_else(|| alias.to_string()), host.port.unwrap_or(22));
    }
    let bare = alias.rsplit('@').next().unwrap_or(alias);
    (bare.to_string(), 22)
}

/// A lightweight reachability check: TCP-connects to the host's ssh port
/// and reports whether it answered and how long that took. This checks
/// network reachability, not that ssh auth would actually succeed.
pub fn check_host(alias: &str) -> HostCheck {
    let (host, port) = resolve_target(alias);
    let addr = match (host.as_str(), port).to_socket_addrs().ok().and_then(|mut a| a.next()) {
        Some(addr) => addr,
        None => {
            return HostCheck {
                reachable: false,
                latency_ms: None,
                error: Some(format!("could not resolve {host}")),
            }
        }
    };

    let start = Instant::now();
    match TcpStream::connect_timeout(&addr, CHECK_TIMEOUT) {
        Ok(_) => HostCheck {
            reachable: true,
            latency_ms: Some(start.elapsed().as_secs_f64() * 1000.0),
            error: None,
        },
        Err(e) => HostCheck {
            reachable: false,
            latency_ms: None,
            error: Some(e.to_string()),
        },
    }
}
