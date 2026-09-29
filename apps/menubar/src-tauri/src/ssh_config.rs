use serde::Serialize;
use std::fs;
use std::net::{TcpStream, ToSocketAddrs};
use std::process::Command;
use std::time::{Duration, Instant};

const CHECK_TIMEOUT: Duration = Duration::from_millis(1500);

/// Default private key filenames ssh tries on its own when no
/// `IdentityFile` is configured for a host — mirrors ssh_config(5)'s
/// built-in `IdentityFile` defaults.
const DEFAULT_IDENTITY_FILES: &[&str] =
    &["id_ed25519", "id_ecdsa", "id_rsa", "id_dsa"];

#[derive(Serialize, Clone, Debug)]
pub struct SshHost {
    pub alias: String,
    pub hostname: Option<String>,
    pub user: Option<String>,
    pub port: Option<u16>,
    /// `IdentityFile` entries for this host, in config order, `~`-expanded.
    pub identity_files: Vec<String>,
}

#[derive(Serialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct HostCheck {
    pub reachable: bool,
    pub latency_ms: Option<f64>,
    pub error: Option<String>,
    /// Set when this host looks like auth will fail outright: no
    /// configured/default identity file exists on disk and no
    /// ssh-agent with loaded keys is reachable either.
    pub auth_warning: Option<String>,
}

fn expand_tilde(path: &str) -> String {
    if let Some(rest) = path.strip_prefix("~/") {
        if let Some(home) = dirs::home_dir() {
            return home.join(rest).to_string_lossy().into_owned();
        }
    }
    path.to_string()
}

/// True if an ssh-agent is reachable and has at least one key loaded.
/// `ssh-add -l` exits 0 with keys listed, 1 with "no identities" if the
/// agent is up but empty, and 2 (or a spawn failure) if there's no agent
/// at all — only the last two cases matter here, both read as "no help".
fn agent_has_keys() -> bool {
    if std::env::var_os("SSH_AUTH_SOCK").is_none() {
        return false;
    }
    Command::new("ssh-add")
        .arg("-l")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

/// Warns when neither a configured/default identity file exists on disk
/// nor an ssh-agent has keys loaded — ssh would have nothing to auth
/// with and the connection will just fail on the remote end.
fn check_auth(identity_files: &[String]) -> Option<String> {
    let candidates: Vec<String> = if identity_files.is_empty() {
        let Some(home) = dirs::home_dir() else {
            return None;
        };
        DEFAULT_IDENTITY_FILES
            .iter()
            .map(|f| home.join(".ssh").join(f).to_string_lossy().into_owned())
            .collect()
    } else {
        identity_files.to_vec()
    };

    if candidates.iter().any(|p| fs::metadata(p).is_ok()) {
        return None;
    }
    if agent_has_keys() {
        return None;
    }

    Some(if identity_files.is_empty() {
        "No default identity file (~/.ssh/id_ed25519, id_rsa, …) and no ssh-agent keys loaded — auth will likely fail.".to_string()
    } else {
        "Configured identity file not found on disk and no ssh-agent keys loaded — auth will likely fail.".to_string()
    })
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
                    identity_files: Vec::new(),
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
            "identityfile" => host.identity_files.push(expand_tilde(value)),
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
    let identity_files = list_hosts()
        .into_iter()
        .find(|h| h.alias == alias)
        .map(|h| h.identity_files)
        .unwrap_or_default();
    let auth_warning = check_auth(&identity_files);

    let (host, port) = resolve_target(alias);
    let addr = match (host.as_str(), port).to_socket_addrs().ok().and_then(|mut a| a.next()) {
        Some(addr) => addr,
        None => {
            return HostCheck {
                reachable: false,
                latency_ms: None,
                error: Some(format!("could not resolve {host}")),
                auth_warning,
            }
        }
    };

    let start = Instant::now();
    match TcpStream::connect_timeout(&addr, CHECK_TIMEOUT) {
        Ok(_) => HostCheck {
            reachable: true,
            latency_ms: Some(start.elapsed().as_secs_f64() * 1000.0),
            error: None,
            auth_warning,
        },
        Err(e) => HostCheck {
            reachable: false,
            latency_ms: None,
            error: Some(e.to_string()),
            auth_warning,
        },
    }
}
