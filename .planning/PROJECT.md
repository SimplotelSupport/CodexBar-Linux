# CodexBar Linux

## What This Is

CodexBar Linux is a native Linux/Ubuntu port of [steipete/CodexBar](https://github.com/steipete/CodexBar) (macOS menubar) and [Finesssee/Win-CodexBar](https://github.com/Finesssee/Win-CodexBar) (Windows system tray). It is a system-tray application that surfaces real-time AI coding-provider usage limits, quota reset countdowns, and cumulative cost so developers can see exactly how much of their AI budget remains without opening a browser.

**Alpha (v0.1.0) ships with 5 providers**: Codex, Claude, OpenAI, Anthropic, OpenRouter. The other 36 providers in the upstream Win-CodexBar crate are deferred to v2 once the trait surface is hardened.

## Core Value

A glanceable Linux tray indicator that tells a developer *right now* how much of their AI provider quota is left and when it resets — without leaving their editor.

## Requirements

### Validated

(None yet — ship to validate)

### Active (v0.1.0-alpha — CLI-only cut)

- [ ] CLI subcommand `codexbar usage -p <provider>` and `codexbar cost -p <provider>` for scripting / CI
- [ ] Codex CLI integration: spawn `codex` and parse local usage/cost from JSONL logs
- [ ] Claude / Gemini CLI fallback integration (parity with macOS PTY runner)
- [ ] Local cost scan from Codex/Claude JSONL logs (last 30 days)
- [ ] Distribution as `.deb` (apt) + `.tar.gz` sidecar for Ubuntu 22.04+ / Debian 12+ on amd64
- [ ] SHA-256 sidecar for each release artifact; GitHub Releases as the distribution channel

### Deferred to v0.2.0 (Tauri tray + popover)

- [ ] Native Linux/Ubuntu tray indicator showing per-provider usage + reset countdown
- [ ] Click-tray popover with provider grid, usage bars, reset times
- [ ] Floating always-on-top usage bar (parity with Win-CodexBar Floating Bar)
- [ ] Settings UI for managing provider credentials (API key, OAuth, browser cookie import)
- [ ] Background polling cadence presets (manual, 1m, 2m, 5m, 15m)
- [ ] Secure secret storage via libsecret (GNOME Keyring / KWallet via Secret Service API)
- [ ] XDG autostart on login (.desktop file in `~/.config/autostart/`)
- [ ] Global hotkey to toggle popover (best-effort under Wayland)
- [ ] AppImage distribution (GUI-app packaging — meaningless without the Tauri shell)
- [ ] Auto-update mechanism (Tauri built-in updater or custom apt repo)
- [ ] Native-feel UI under GNOME (libadwaita styling) and KDE Plasma 5/6
- [ ] Wayland + X11 dual support; graceful degradation when tray unavailable

### Deferred to v0.3.0

- [ ] Remaining ~36 providers from upstream Win-CodexBar (Cursor, Gemini, Mistral, etc.)

### Out of Scope

- Snap package — Snap confinement breaks system-tray and global-shortcut interfaces reliably; flatpak/deb give better UX
- Browser cookie auto-import from Chrome/Firefox profiles — Linux lacks unified Keychain equivalent; users will paste cookies or use API keys/OAuth instead (defer to v2 if demand)
- macOS Keychain / Windows DPAPI parity — Linux uses Secret Service API; not a 1:1 port
- WidgetKit-style desktop widgets — no GNOME/KDE equivalent that's worth maintaining for v1
- Sparkle-style delta auto-update — apt and AppImage's native updater are simpler and idiomatic
- 32-bit support — Ubuntu LTS dropped i386 desktop; arm64 + amd64 only
- Distros older than Ubuntu 22.04 / Debian 12 — webview2/webkitgtk version requirements
- Custom skinning / theming beyond system theme inheritance
- Mobile / Android port

## Context

- **Reference repos**: [steipete/CodexBar](https://github.com/steipete/CodexBar) (Swift/SwiftUI + AppKit, Sparkle, KeyboardShortcuts, 29+ providers) and [Finesssee/Win-CodexBar](https://github.com/Finesssee/Win-CodexBar) (Tauri v2 + Rust + React, 41+ providers, Inno Setup installer)
- **Windows port already uses Tauri v2 + Rust** — the Rust backend crate (`codexbar`) is shared, which makes a Linux Tauri target the lowest-friction approach (Tauri v2 ships first-class Linux support via webkitgtk)
- **Linux desktop fragmentation**: GNOME 45+ has degraded AppIndicator support; KDE Plasma uses StatusNotifierItem (DBus) reliably; Wayland tray is improving but imperfect — design must detect DE/session at runtime
- **Target user**: developer on Ubuntu 22.04+ / Debian 12+ / Fedora 39+ / Arch with `codex`, `claude`, or other AI CLIs already installed, hitting daily/weekly quota walls and wanting visibility
- **Prior art on Linux**: nothing equivalent ships today — closest are paid SaaS dashboards or per-provider CLIs; this is a greenfield Linux opportunity even though Mac/Windows are solved

## Constraints

- **Tech stack**: Tauri v2 + Rust backend (reuse Windows `codexbar` crate where possible) + webkitgtk-rendered frontend (React/TS to match Win-CodexBar, or Svelte if friction with React-on-Linux Tauri build) — locks us in to webkitgtk 6.0+ which means Ubuntu 22.04 minimum
- **Tray**: StatusNotifierItem via DBus is the primary; AppIndicator + libayatana-appindicator3 as fallback for GNOME; degrade gracefully to a menu-less window if neither is available
- **Secrets**: Secret Service API via `secret-service` Rust crate (libsecret-backed) — no plaintext API keys on disk by default; offer encrypted-file fallback if no Secret Service is running (e.g., headless sessions)
- **Distribution**: `.deb` and AppImage as v1 targets; Flatpak is v2 (sandboxing + portal complexity)
- **Performance**: tray icon + background poller must idle at <50 MB RSS and <0.5% CPU when no popover is open
- **Timeline**: aggressive — user has asked for PRD → dev → testing → prod; expect roadmap to fit a 4–6 week build
- **No paid CI**: dev/test must run on free GitHub Actions; AppImage and `.deb` build must be CI-friendly (no codesigning hardware)
- **Security**: every secret path through Secret Service or encrypted-at-rest; no telemetry by default; opt-in crash reporting only

## Key Decisions

| Decision | Rationale | Outcome |
|----------|-----------|---------|
| Use Tauri v2 + Rust (not Electron, not native GTK4) | Reuses Windows port's Rust backend crate; ~100 MB lighter than Electron; first-class Tauri Linux support; same frontend can run all 3 OSes | — Pending validation in Phase 1 |
| **Selectively vendor headless modules from Win-CodexBar (not fork-and-feature-gate)** | Win-CodexBar's `codexbar` crate has GUI deps as unconditional `[dependencies]` (winit, eframe, tray-icon, muda, global-hotkey, keyring, aes-gcm). Feature-gating means modifying upstream; selective vendor = delete what we don't need, keep MIT attribution, zero GUI baggage. Cleaner than carrying a fork in lockstep. | — Locked 2026-05-17 |
| **Alpha (v0.1.0) scope: 5 providers** — Codex, Claude, OpenAI CLI, Anthropic API (via `openaiapi/` shape), OpenRouter | Codex + Claude exercise the JSONL cost scanner; the other 3 exercise clean HTTP+rustls. 41 providers in v1 is fantasy; rest are tracked as v2. Forces every cross-cutting concern (auth, errors, parsing) through 5 distinct shapes, which is enough to harden the trait | — Locked 2026-05-17 |
| MIT attribution preserved | Win-CodexBar is MIT-licensed; vendored source must retain original LICENSE alongside it (`crates/codexbar-core/LICENSE-WIN-CODEXBAR`) + top-level NOTICE crediting upstream | — Locked 2026-05-17 |
| **v0.1.0-alpha = CLI-only (`.deb` + `.tar.gz`)** | Tauri shell + tray + popover is a multi-week build. Ship the CLI alpha tonight to validate the headless core in the wild, gather feedback on the 5-provider surface, then layer the GUI in v0.2.0. AppImage is deferred since it's a GUI-app format — `.tar.gz` is the idiomatic CLI sidecar. | — Locked 2026-05-17 |
| Target Ubuntu 22.04 LTS + Debian 12 as v1 baseline | Covers ~80% of dev-Linux desktops; matches webkitgtk 6.0 floor; older distros routed to AppImage | — Pending |
| `.deb` + AppImage for v1; Flatpak deferred | apt is idiomatic on Ubuntu; AppImage covers everything else; Flatpak adds portal/sandbox engineering cost we cannot absorb in v1 | — Pending |
| Secret Service API for credentials, encrypted-file fallback | Standard freedesktop spec; works on GNOME Keyring + KWallet; encrypted fallback covers headless / unsupported sessions | — Pending |
| Skip browser-cookie auto-import for v1 | Mac (Keychain) and Win (DPAPI) implementations are platform-specific; Linux equivalents are fragile across distros — defer until v2 with demand signal | — Pending |
| PRD → dev → testing → prod phase shape, coarse granularity | User explicitly requested this flow; coarse phases (3–5) keep us moving fast | — Pending |

## Evolution

This document evolves at phase transitions and milestone boundaries.

**After each phase transition** (via `/gsd-transition`):
1. Requirements invalidated? → Move to Out of Scope with reason
2. Requirements validated? → Move to Validated with phase reference
3. New requirements emerged? → Add to Active
4. Decisions to log? → Add to Key Decisions
5. "What This Is" still accurate? → Update if drifted

**After each milestone** (via `/gsd-complete-milestone`):
1. Full review of all sections
2. Core Value check — still the right priority?
3. Audit Out of Scope — reasons still valid?
4. Update Context with current state

---
*Last updated: 2026-05-17 — scope cut to v0.1.0-alpha = CLI-only; Tauri shell deferred to v0.2.0*
