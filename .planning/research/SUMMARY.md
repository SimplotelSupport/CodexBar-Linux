# Project Research Summary

**Project:** CodexBar Linux
**Domain:** Tauri v2 + Rust Linux system-tray desktop app (AI provider usage/quota monitor)
**Researched:** 2026-05-17
**Confidence:** HIGH

## Executive Summary

CodexBar Linux is a platform port, not a greenfield build. The Windows port (Tauri v2 + Rust) already has a working `codexbar` Rust crate with 41 provider implementations, a defined `Provider` trait, clap CLI subcommands, and a React/TS/Vite frontend. The recommended approach: feature-gate the Windows GUI dependencies (`eframe`, `winit`, `tray-icon`) behind a Cargo feature flag so the core crate compiles headless on Linux, add a thin Tauri Linux shell that reuses the core crate verbatim, and distribute as `.deb` + AppImage for Ubuntu 22.04+ / Debian 12+. The only net-new code is the Linux platform layer: `TrayBridge` (SNI + AppIndicator fallback detection), `SecretStore` (`oo7` crate — Secret Service primary, encrypted-file fallback), and Tauri plugin wiring.

The central risk is Linux desktop fragmentation. GNOME 45+ dropped native AppIndicator; Wayland blocks global hotkeys and always-on-top windows for most compositors; webkitgtk versions differ between Ubuntu 22.04 (4.1) and 24.04 (6.0). None are showstoppers but each requires runtime detection and graceful degradation rather than silent failure. Ship X11 as fully-supported; document the Wayland partial-feature matrix.

The single hardest blocker is Phase 0: the `codexbar` crate currently compiles GUI deps unconditionally, which conflicts with Tauri's own copies of `winit` and `tray-icon`. This must be resolved before any Tauri code is written.

---

## Key Findings

### Stack at a Glance

| Layer | Choice | Rationale |
|-------|--------|-----------|
| Desktop framework | Tauri v2 | Windows port already on it; reuses frontend + Rust crate; ~15 MB vs ~150 MB Electron |
| Rust core | `codexbar` crate (forked, feature-gated) | 41 provider impls, Provider trait, CLI — reuse verbatim |
| Frontend | React 18 + TypeScript + Vite + Tailwind | Copy from Windows port unchanged |
| Secret storage | `oo7` 0.4.x | Secret Service DBus primary + XChaCha20-Poly1305 file fallback; better than `keyring-rs` |
| HTTP | `reqwest` + `rustls-tls` | Avoids OpenSSL ABI drift across Ubuntu/Fedora/Arch |
| PTY | `portable-pty` | CLI provider runners (codex, claude interactive sessions) |
| Tray (primary) | `tauri::tray::TrayIconBuilder` | SNI/StatusNotifierItem on KDE/MATE/Cinnamon |
| Tray (GNOME fallback) | `libayatana-appindicator3` | GNOME 45+ needs this system package or extension |
| Single-instance | `tauri-plugin-single-instance` | DBus name lock; must register FIRST in plugin chain |
| XDG paths | `directories` 5.x | Never hardcode `~/.config` |
| Distribution | `.deb` + AppImage | Snap excluded (tray broken under confinement); Flatpak deferred |

### Table Stakes vs Differentiators vs Anti-Features

**Table stakes (v1 must-ship):**
- Tray icon + click popover with per-provider usage grid and reset countdown
- Background poller (manual/1m/2m/5m/15m cadence)
- Credentials UI backed by Secret Service (encrypted-file fallback)
- Codex/Claude/Gemini/Copilot/OpenRouter provider plugins
- Local JSONL cost scan from `~/.codex/` and `~/.claude/`
- `codexbar usage/cost -p <provider>` CLI subcommands
- XDG autostart `.desktop`, single-instance DBus lock
- `.deb` + AppImage distribution; crash-free degradation when tray host absent

**Differentiators (v1 nice-to-have):**
- Floating always-on-top usage bar (X11 full; KDE Wayland partial; GNOME Wayland degrades gracefully)
- Global hotkey (X11 reliable; Wayland best-effort — never block release on it)
- Provider quota notifications via DBus toast (80%/100%)
- Auto-update via Tauri signed updater manifest
- 30-day rolling cost view in popover

**Defer to v2:** OAuth per provider, browser cookie import, Flatpak, arm64, custom apt repo, per-provider rate-limit backoff, telemetry, i18n

**Anti-features (explicitly not building):** Snap packaging, 32-bit i386, custom theming engine, XEmbed legacy tray, WidgetKit-equivalent desktop widgets, telemetry-by-default

### Architecture in One Paragraph

The architecture is strictly "headless core + platform shell." The `codexbar` Rust crate (`rust/`) has zero Tauri or GTK imports — only HTTP, PTY, JSON, XDG paths. The Tauri crate (`apps/desktop-linux/src-tauri/`) is a thin shell: `TrayBridge` for DE detection and tray icon, `SecretStore` (`oo7` wrapper), `AppState` (`Mutex<HashMap<ProviderId, UsageSnapshot>>`), and `#[tauri::command]` IPC handlers. GTK main thread owns all window operations; all provider fetches run on `tauri::async_runtime::spawn` (never raw `tokio::spawn` — causes "no reactor running" panic in window listeners); filesystem scans use `spawn_blocking`. The `app.emit("usage-updated", payload)` event drives React re-renders.

```
UI Layer (React/TS — webkitgtk WebView, copied from Windows port)
  Popover | Floating Bar | Settings Window
────────────────────────────────────────────────────
Tauri IPC Bridge  (#[tauri::command], emit events)
  TrayBridge | commands/* | SecretStore | FloatBar | ShortcutBridge
────────────────────────────────────────────────────
Rust Core  (codexbar crate — ZERO Tauri imports)
  Provider trait + 41 impls | cost_scanner | settings | clap CLI
────────────────────────────────────────────────────
Linux Platform Layer  (new code)
  SNI/AppIndicator detect | oo7 (DBus + enc. file) | single-instance DBus
────────────────────────────────────────────────────
External
  41 AI provider APIs | codex/claude CLIs (PTY) | JSONL logs | GNOME Keyring/KWallet
```

### Top 5 Must-Mitigate Pitfalls (with phase mapping)

| # | Pitfall | Prevention | Phase |
|---|---------|------------|-------|
| P-01 | `codexbar` crate pulls `eframe`/`winit`/`tray-icon` unconditionally — conflicts with Tauri's copies | Feature-gate all GUI deps behind `gui-egui` Cargo feature (off by default); verify with `cargo tree` | Phase 0 — BLOCKER |
| P-02 | GNOME 45+ tray icon invisible (AppIndicator extension not installed) | Bundle `gnome-shell-extension-appindicator` as Recommends in `.deb`; detect missing tray at startup and surface actionable message | Phase 2 + 4 |
| P-03 | Wayland silently kills hotkey + floating bar | Detect `XDG_SESSION_TYPE`; mark as best-effort in UI; never panic on registration failure | Phase 2 |
| P-04 | Secret Service absent on minimal/headless Ubuntu installs | `oo7` encrypted-file fallback — wire from day one; show active backend in Diagnostics | Phase 3 |
| P-05 | `.deb` built outside Ubuntu 22.04 CI has wrong runtime deps | Always build in `ubuntu-22.04` GHA runner; verify `dpkg -I` shows `libwebkit2gtk-4.1-0` | Phase 4 |

Honorable mentions: P-06 (rate limit exhaustion — per-provider backoff + circuit breaker), P-07 (JSONL scanner blocks main thread — use `spawn_blocking` + mtime-keyed cache), P-15 (use `rustls-tls` not `native-tls` — set in Phase 0 Cargo manifest).

---

## Implications for Roadmap

**Suggested phases: 5 (Phase 0 + Phase 1–4)**

### Phase 0: PRD + Crate Strategy + Workspace Scaffold
**Rationale:** P-01 blocks everything. Resolve the Cargo conflict before writing a single line of Tauri code. PRD-level decisions captured.
**Delivers:** PRD document; Cargo workspace with feature-gated `codexbar` crate; `cargo build -p codexbar` green on Linux headlessly; `reqwest` pinned to `rustls-tls`
**Avoids:** P-01, P-15

### Phase 1: CLI + Headless Core on Linux
**Rationale:** Validate provider pipeline runs on Linux before any UI. Audit 41 providers for Windows-specific I/O (registry, DPAPI). Get `codexbar usage/cost` CLI working.
**Delivers:** All 41 providers compile + test green (mockito fixtures); `codexbar` CLI binary on Linux; log rotation configured
**Avoids:** P-14 (log rotation), P-10 (provider parse fixture tests)

### Phase 2: Tauri Shell + Tray + Popover (Dev)
**Rationale:** Data pipeline exists; add the visible layer. Tray DE detection is highest-risk — spike GNOME 45 + KDE Plasma 6 early to validate P-02/P-03 assumptions before building more UI.
**Delivers:** App launches; tray visible on GNOME + KDE; click opens popover WebView; single-instance DBus lock; degraded window mode when tray absent
**Avoids:** P-02, P-03, P-13 (hotkey init race), P-19 (tray renders before wizard)

### Phase 3: Data Pipeline + Credentials + Settings (Dev → Testing)
**Rationale:** Shell exists; wire the data. Credentials needed before provider fetches succeed in production; JSONL scanner, rate-limit backoff, snapshot cache all belong here. End-to-end test matrix executed.
**Delivers:** Live usage data in popover; settings window with API key storage (Secret Service + file fallback); XDG snapshot cache (sub-100ms restart); autostart `.desktop`; 30-day cost scan; CI integration tests green
**Avoids:** P-04, P-06, P-07, P-09

### Phase 4: Polish + Distribution + Prod Release
**Rationale:** App works end-to-end; now make it shippable. Floating bar + hotkey are Wayland-constrained polish. CI packaging matrix is concrete work. Tagged production release.
**Delivers:** `.deb` + AppImage from `ubuntu-22.04` CI; auto-update; floating bar (X11 full); global hotkey (X11/best-effort); GNOME/KDE theme follow; GNOME extension as Recommends; GA release with SHA-256 sidecars
**Avoids:** P-05, P-08 (webkit 4.1 vs 6.0 matrix), P-11, P-12, P-20

### Phase Ordering Rationale

- Phase 0 before all code: crate fork is a structural decision, not a feature; resolving it unblocks headless CI for all subsequent phases.
- Phase 1 before Phase 2: all `cargo test -p codexbar` runs headlessly; display server not needed until Tauri shell.
- Credentials in Phase 3 (not 2): placeholder config works for Phase 2 tray spike; deferring Secret Service keeps Phase 2 focused on the highest-risk DE compatibility question.
- Distribution last: no value optimizing packaging until app is functionally complete.

---

## Confidence Assessment

| Area | Confidence | Notes |
|------|------------|-------|
| Stack | HIGH | Win-CodexBar Cargo.toml read directly; crate versions verified |
| Features | HIGH | Classified against both reference repos; Linux-specific features identified |
| Architecture | HIGH | Windows source read directly; Tauri v2 docs verified; GNOME tray regression is MEDIUM |
| Pitfalls | HIGH | Validated against Tauri issue tracker + Linux desktop spec landscape |

**Overall: HIGH**

### Open Questions

1. **GNOME 45+ AppIndicator behavior varies by distro patch** — confirmed community reports, but exact behavior on stock Ubuntu 22.04 vs 24.04 vs Fedora 39 needs VM validation in Phase 2 before finalizing detection logic.
2. **webkitgtk 4.1 vs 6.0 packaging strategy** — two Ubuntu LTS versions ship different webkit majors; single binary (pin to 4.1) or two `.deb` variants needs a concrete decision before Phase 4 CI.
3. **Wayland GlobalShortcut portal** — no stable implementation as of 2026-05-17; treat as unavailable for v1, revisit in v2.
4. **41 provider Windows-specific I/O scope** — number of providers using registry reads or DPAPI is unknown until Phase 1 audit; could be 0 or 10+.

---

*Research completed: 2026-05-17. Ready for roadmap.*
