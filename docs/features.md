# Mapper — Feature TODO

Brainstormed list of possible features. Nothing here is committed to —
review and mark what you actually want, drop the rest. Implementation
starts only after you confirm.

## Core forwarding

- [ ] Persist active tunnels across app restart (re-launch saved forwards on start)
- [ ] Remote forwarding (`-R`) in addition to local (`-L`)
- [ ] Dynamic/SOCKS forwarding (`-D`)
- [ ] Multiple simultaneous forwards through one ssh connection (multiplex, `-J`/`ControlMaster`)
- [ ] Auto-reconnect a tunnel if it drops (retry with backoff)
- [ ] Named/saved forward presets ("staging db", "prod redis") instead of re-entering host/ports each time
- [ ] Edit an existing forward without stopping/recreating it
- [ ] Duplicate a running forward with one click

## SSH config

- [ ] Support `Include` directives in `~/.ssh/config` (multi-file configs)
- [ ] Support `ProxyJump`/`ProxyCommand` hosts
- [ ] Warn if selected host has no identity file / relies on agent with no keys loaded
- [ ] Manual host entry mode toggle (bypass ~/.ssh/config entirely, already partially there)
- [ ] Edit ~/.ssh/config from within the app

## Menu bar / UX

- [ ] Show active tunnel count as a badge/text next to the tray icon
- [ ] Menu-bar dropdown list of active tunnels (quick view without opening the window)
- [ ] Global keyboard shortcut to open/close the popover
- [ ] Launch at login toggle
- [ ] Light/dark tray icon auto-switching refinement (already using template icon — verify on all wallpapers)
- [ ] Notifications (macOS notification center) when a tunnel dies unexpectedly
- [ ] Sound/visual indicator on tunnel failure

## Reliability / diagnostics

- [x] Per-tunnel live status (connecting / connected) — probes the local port each poll; no "retrying" state since that implies auto-reconnect, which isn't built
- [x] Show ssh stderr/log tail for a running tunnel, not just on death — background reader keeps a 200-line ring buffer, viewable via the per-tunnel "Log" toggle
- [x] Latency/health check ping on the forwarded port — same probe reports round-trip ms next to the host
- [ ] Configurable ssh options per tunnel (ServerAliveInterval, compression, etc.) instead of fixed defaults
- [x] Detect and warn about stale/orphaned ssh processes from a previous crashed run — scanned on launch, shown as a banner with a kill-all action

## Data & sync

- [ ] Persist saved tunnel presets to disk (JSON) across app updates
- [ ] Export/import tunnel presets (share a set of forwards with teammates)
- [ ] iCloud/file sync of presets across machines

## Security

- [ ] Confirm before killing a process on "kill port & retry" (show what's using it first)
- [ ] Passphrase-protected key support (prompt via macOS Keychain instead of failing BatchMode)
- [ ] Audit log of tunnels started/stopped with timestamps

## Packaging / distribution

- [ ] Code-sign + notarize for distribution outside dev machine
- [ ] Auto-update (Tauri updater)
- [ ] Homebrew cask
