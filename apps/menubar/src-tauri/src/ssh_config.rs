use serde::Serialize;
use std::fs;

#[derive(Serialize, Clone, Debug)]
pub struct SshHost {
    pub alias: String,
    pub hostname: Option<String>,
    pub user: Option<String>,
    pub port: Option<u16>,
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
