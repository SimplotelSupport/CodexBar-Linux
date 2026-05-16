# Pitfalls Research

**Domain:** Linux/Ubuntu Tauri v2 + Rust tray app (AI provider usage monitor)
**Researched:** 2026-05-17
**Confidence:** HIGH (validated against Tauri issues + Win-CodexBar source + Linux desktop spec landscape)

---

## Critical Pitfalls (will sink the project if missed)

### P-01 — `codexbar` crate over-pulls GUI dependencies

- **Warning signs:** Linking `tauri` and `codexbar` together blows up with `winit` / `tray-icon` / `muda` / `global-hotkey` / `eframe` / `egui` symbol conflicts.
- **Cause:** Win-CodexBar's `rust/Cargo.toml` lists those as unconditional `[dependencies]`. Tauri v2 ships its own copies and won't coexist.
- **Prevention:** Phase 0 must feature-gate every GUI crate behind a `gui-egui` Cargo feature (off by default), or fork the crate stripping GUI dependencies entirely. Verify with `cargo tree -p codexbar` showing zero `winit`/`tray-icon` paths when Tauri is the consumer.
- **Phase:** Phase 0 (foundation) — blocker for everything else.

### P-02 — GNOME 45+ has degraded AppIndicator

- **Warning signs:** Tray icon invisible on default Ubuntu 24.04 GNOME Wayland session, but works on KDE / XFCE / Cinnamon.
- **Cause:** GNOME Shell dropped legacy systray; AppIndicator survives only via the user-installed `gnome-shell-extension-appindicator`.
- **Prevention:** Bundle `gnome-shell-extension-appindicator` as a **Recommends** in `.deb`; document the manual install for AppImage users; detect missing tray host at runtime and surface a clear actionable message. Test under stock Ubuntu 22.04 + 24.04 GNOME sessions.
- **Phase:** Phase 2 (tray + UI) and revisited in Phase 4 (distribution).

### P-03 — Wayland kills X11-only features silently

- **Warning signs:** Global hotkey or always-on-top floating bar works in dev (X11) but is a no-op on user machines (Wayland).
- **Cause:** Wayland mandates portals for cross-window input/positioning; xdg-desktop-portal has no GlobalShortcut portal in stable yet, and `_NET_WM_STATE_ABOVE` is honored by KWin but not GNOME Mutter.
- **Prevention:** Feature-detect session type (`XDG_SESSION_TYPE`); when Wayland, mark hotkey + floating-bar as "best-effort" in UI and surface a once-per-session info banner; never panic on registration failure.
- **Phase:** Phase 2 (UI) — design hotkey/floating-bar as optional, not blocking.

### P-04 — Secret Service absent in headless / minimal sessions

- **Warning signs:** First-run on a server-style Ubuntu install (no gnome-keyring-daemon) fails to save credentials.
- **Cause:** `oo7`'s DBus path requires a running keyring daemon; many minimal installs lack one.
- **Prevention:** Use `oo7`'s built-in encrypted-file fallback (`oo7::file::Keyring`) when DBus is unavailable. Surface which backend is in use in Settings → Diagnostics.
- **Phase:** Phase 3 (credentials).

### P-05 — `cargo tauri build` produces broken `.deb` on non-Ubuntu hosts

- **Warning signs:** `.deb` built on Fedora installs but crashes on Ubuntu due to mismatched webkitgtk version or wrong dependency names.
- **Cause:** cargo-deb derives runtime deps from the host system's package metadata, not the target.
- **Prevention:** Always build `.deb` in CI on the exact `ubuntu-22.04` runner image; verify `dpkg -I codexbar.deb` lists `libwebkit2gtk-4.1-0`, `libgtk-3-0`, `libayatana-appindicator3-1` as runtime deps. Do not ship `.deb` artifacts built on developer Macs.
- **Phase:** Phase 4 (distribution).

## High-Impact Pitfalls

### P-06 — Background poller exhausts provider rate limits

- **Warning signs:** 429 cascades after the app is open for hours; user IP soft-banned by a provider.
- **Cause:** Default 1-minute cadence × N enabled providers × always-on = many calls/day.
- **Prevention:** Per-provider `min_interval` baked into each plugin's manifest; respect `Retry-After`; exponential backoff with jitter on consecutive failures; circuit-breaker after 5 sequential failures pauses that provider until next user action.
- **Phase:** Phase 3 (data pipeline).

### P-07 — JSONL cost scanner walks giant directories

- **Warning signs:** UI freezes on first popover open; high disk I/O.
- **Cause:** Walking `~/.codex/sessions/` synchronously on the main thread when users have months of history.
- **Prevention:** Run scan on a tokio blocking task; cache the result with a TTL keyed on directory mtime; show a "scanning…" placeholder.
- **Phase:** Phase 3.

### P-08 — Webkitgtk version mismatch between Ubuntu 22.04 and 24.04

- **Warning signs:** App builds on 24.04 but won't load WebView on 22.04 (missing `libwebkitgtk-6.0-4`).
- **Cause:** Ubuntu 24.04 ships webkit2gtk-6.0; 22.04 ships webkit2gtk-4.1. Tauri v2 supports both via feature flags.
- **Prevention:** Build matrix in CI: both 22.04 and 24.04 runners; ship two `.deb` variants (`-4.1` and `-6.0` suffix) or pin to 4.1 for v1 to cover both.
- **Phase:** Phase 4 (distribution).

### P-09 — `XDG_RUNTIME_DIR` missing causes DBus failures

- **Warning signs:** Single-instance lock or Secret Service silently fails when launched from systemd user service.
- **Cause:** Some launch contexts strip XDG runtime env.
- **Prevention:** Detect and surface a clear error pointing to systemd unit env setup; document in README.
- **Phase:** Phase 4.

### P-10 — Provider plugin contributors break the wire format

- **Warning signs:** Crashes parsing a provider response after a third-party PR.
- **Cause:** Permissive `serde_json::Value` parsing accepted upstream API shifts that violated our `UsageSnapshot` contract.
- **Prevention:** Strongly-typed `serde::Deserialize` per provider response; mockito fixture tests for every provider; CI gate that runs all provider parsers against checked-in fixtures.
- **Phase:** Phase 3.

## Medium-Impact Pitfalls

### P-11 — AppImage auto-update collides with system updates

- **Warning signs:** Two versions in `~/Applications/`; user confused which is "current".
- **Cause:** AppImage updates copy alongside instead of replacing.
- **Prevention:** Use `appimageupdate` flow (sidecar file); document; deprioritize AppImage auto-update vs apt repo.
- **Phase:** Phase 4.

### P-12 — `.desktop` autostart written before binary install path is finalized

- **Warning signs:** Autostart entry's `Exec=` points to a stale path after upgrade.
- **Cause:** Hardcoded path in `.desktop` file.
- **Prevention:** Use `Exec=codexbar` (PATH-resolved) when installed via `.deb`; use absolute path for AppImage but rewrite on every launch.
- **Phase:** Phase 4.

### P-13 — `tauri-plugin-global-shortcut` registration race on app start

- **Warning signs:** Hotkey not registered on cold start, works after a settings save.
- **Cause:** Plugin initialized before main window is ready.
- **Prevention:** Register hotkeys in `app.on_window_event(WindowEvent::CloseRequested)`-bracketed initialization; not in `setup` callback before window creation.
- **Phase:** Phase 2.

### P-14 — Logger writes inside `XDG_STATE_HOME` without rotation

- **Warning signs:** `~/.local/state/codexbar/logs/` grows multi-GB over time.
- **Cause:** `tracing-subscriber` writes append-only by default.
- **Prevention:** Use `tracing-appender::rolling::daily()` with retention; cap at 7 days.
- **Phase:** Phase 1.

### P-15 — Reqwest with system OpenSSL causes Fedora vs Ubuntu ABI drift

- **Warning signs:** `.deb` built on Ubuntu segfaults on Fedora due to OpenSSL major version mismatch.
- **Cause:** `native-tls` links system OpenSSL.
- **Prevention:** Use `reqwest = { features = ["rustls-tls"], default-features = false }`. Don't enable `native-tls`.
- **Phase:** Phase 0 (Cargo manifest setup).

## Low-Impact Pitfalls (good hygiene)

| ID | Pitfall | Prevention |
|----|---------|------------|
| P-16 | Tray icon doesn't update glyph until popover opens | Use `set_icon()` from the poller task via Tauri's `AppHandle` |
| P-17 | Popover positioned off-screen on multi-monitor setups | Use `tauri::window::PhysicalPosition` derived from cursor + tray bounds |
| P-18 | Floating-bar position resets after monitor hotplug | Persist position; restore in `on_window_event(WindowEvent::Moved)` debounced |
| P-19 | First-run wizard blocks tray icon from appearing | Render tray icon first; surface first-run prompt only via popover |
| P-20 | Spelling of "AppIndicator" in `.deb` deps | Package name is `libayatana-appindicator3-1`, not `libappindicator3` (deprecated Ubuntu fork) |

## Cross-Reference to Phases

| Phase | Pitfalls to mitigate |
|-------|----------------------|
| Phase 0 (PRD + foundation) | P-01, P-15 |
| Phase 1 (CLI works on Linux) | P-14 |
| Phase 2 (Tauri shell + tray + UI) | P-02, P-03, P-13, P-16, P-17, P-18, P-19 |
| Phase 3 (data pipeline) | P-04, P-06, P-07, P-10 |
| Phase 4 (distribution + testing + prod) | P-05, P-08, P-09, P-11, P-12, P-20 |

## What Linux ports of Mac/Win tray apps commonly get wrong

1. **Assuming X11 is the default** — most user machines in 2026 are Wayland; design Wayland-first.
2. **Hardcoding GNOME** — KDE Plasma has the biggest Linux gamer/dev share; never test only on one DE.
3. **Plain-text secrets** — "I'll add Secret Service later" never happens; ship it from day one.
4. **Bundling wrong libs** — Snap and Flatpak sandbox tray icons; deb + AppImage is the path of least resistance.
5. **Skipping `.desktop` MimeType / Categories** — keeps app out of GNOME search and KDE KRunner.
6. **Forgetting `OnlyShowIn=` policy on autostart** — being polite means not auto-launching headless servers.
7. **Telemetry by default** — Linux users churn instantly when they catch unannounced telemetry.
