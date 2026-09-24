# Mapper — Feature TODO

Brainstormed list of possible features. Nothing here is committed to —
review and mark what you actually want, drop the rest. Implementation
starts only after you confirm.

## Core forwarding

- [x] Persist active tunnels across app restart (re-launch saved forwards on start) — every tunnel has a persisted `running` intent; app launch restarts anything marked running, quit/kill always kills the OS processes but leaves that intent untouched so it comes back next launch. A tunnel that dies unexpectedly (crash, auth failure) flips to stopped instead of crash-looping
- [ ] Remote forwarding (`-R`) in addition to local (`-L`)
- [ ] Dynamic/SOCKS forwarding (`-D`)
- [ ] Multiple simultaneous forwards through one ssh connection (multiplex, `-J`/`ControlMaster`)
- [x] Auto-reconnect a tunnel if it drops (retry with backoff) — capped exponential backoff (2s→4s→8s→16s→30s), gives up after 6 consecutive failures rather than retrying a permanently dead host forever; surfaces as a "Retrying" status with a countdown, and finally becomes a failure (shown in the banner) once it gives up
- [x] Named/saved forwards — every started tunnel is saved automatically (deduped by connection), no separate "preset" concept or manual save step; stopping a tunnel keeps it in the list so it's a one-click Start away, Delete removes it for good
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

- [x] Per-tunnel live status (connecting / connected / retrying) — probes the local port each poll; retrying added alongside auto-reconnect below
- [x] Show ssh stderr/log tail for a running tunnel, not just on death — background reader keeps a 200-line ring buffer, viewable via the per-tunnel "Log" toggle
- [x] Latency/health check ping on the forwarded port — same probe reports round-trip ms next to the host
- [ ] Configurable ssh options per tunnel (ServerAliveInterval, compression, etc.) instead of fixed defaults
- [x] Detect and warn about stale/orphaned ssh processes from a previous crashed run — scanned on launch, shown as a banner with a kill-all action

## Data & sync

- [x] Persist saved tunnels to disk (JSON) across app updates — written to the app's config dir as `tunnels.json`, reloaded on launch
- [x] Export/import saved tunnels (share a set of forwards with teammates) — native save/open dialog, plain JSON file; import merges by connection (host + all three ports), skipping duplicates, and always comes in stopped
- [ ] iCloud/file sync across machines — explicitly held off per your instruction, not attempted

## Security

- [ ] Confirm before killing a process on "kill port & retry" (show what's using it first)
- [ ] Passphrase-protected key support (prompt via macOS Keychain instead of failing BatchMode)
- [ ] Audit log of tunnels started/stopped with timestamps

## Packaging / distribution

- [ ] Code-sign + notarize for distribution outside dev machine
- [ ] Auto-update (Tauri updater)
- [ ] Homebrew cask
