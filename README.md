<p align="center">
  <img src="apps/menubar/src-tauri/icons/128x128.png" alt="Mapper logo" width="96" />
</p>
<p align="center">The macOS menu bar app for SSH port forwards.</p>
<p align="center">
  <a href="LICENSE"><img alt="License" src="https://img.shields.io/badge/license-MIT-blue.svg?style=flat-square" /></a>
  <a href="#"><img alt="Platform" src="https://img.shields.io/badge/platform-macOS-lightgrey.svg?style=flat-square" /></a>
</p>

<p align="center">
  <img src="demo.gif" alt="Mapper demo" width="420" style="border-radius: 12px;" />
</p>

Pick a host from your SSH config, set a local and remote port, and Mapper keeps the tunnel alive in the background — no terminal commands, no tmux sessions.

---

## Features

- Start, stop, and manage local SSH port forwards (`-L`) from the menu bar
- Named/saved tunnels persisted to disk and restarted on launch
- Auto-reconnect with capped exponential backoff (2s up to 30s, gives up after 6 failures)
- Per-tunnel live status (connecting / connected / retrying) with round-trip latency
- In-app SSH log viewer (200-line ring buffer per tunnel)
- Stale/orphaned process detection with one-click kill
- Export and import saved tunnels as plain JSON
- Adjustable ping poll interval for status and latency refresh

## Installation

Download the latest `Mapper.dmg` from the releases page.

> [!NOTE]
> Requires macOS with an OpenSSH client (ships with the OS). No server installs required.

## Usage

- Add a tunnel from the New Tunnel form: choose a host from `~/.ssh/config` or type one directly, set the local and remote ports, and start.
- The tunnel is saved automatically, deduplicated by connection. Stopping keeps it in the list for a one-click restart; Delete removes it for good.
- Each running tunnel shows live status, latency, and a log viewer.
- If a local port is already in use, "kill port & retry" frees it and restarts.
- Tunnels marked running relaunch on app start. One that dies unexpectedly flips to stopped instead of crash-looping.

## Data & storage

- Saved tunnels live in `tunnels.json` in `~/Library/Application Support/dev.jabed.mapper/`.
- Imports merge by connection, skip duplicates, and always come in stopped.

## Documentation

For the full feature list and roadmap, see [docs/features.md](docs/features.md).

## Contributing

If you are interested in contributing to Mapper, read the [docs/features.md](docs/features.md) roadmap first, then open a pull request. Commit messages follow the conventional format and must include a scope, for example `feat(menubar): ...`.

## Security

If you find a security issue, please do not open a public issue. Report it privately via GitHub's security advisories so it can be addressed before disclosure.

## License

MIT — see [LICENSE](LICENSE) for details.