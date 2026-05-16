<!-- GSD:project-start source:PROJECT.md -->
## Project

**CodexBar Linux**

CodexBar Linux is a native Linux/Ubuntu port of [steipete/CodexBar](https://github.com/steipete/CodexBar) (macOS menubar) and [Finesssee/Win-CodexBar](https://github.com/Finesssee/Win-CodexBar) (Windows system tray). It is a system-tray application that surfaces real-time AI coding-provider usage limits, quota reset countdowns, and cumulative cost across 30+ providers (Codex, Claude, Cursor, Gemini, Copilot, OpenRouter, etc.) so developers can see exactly how much of their AI budget remains without opening a browser.

**Core Value:** A glanceable Linux tray indicator that tells a developer *right now* how much of their AI provider quota is left and when it resets — without leaving their editor.

### Constraints

- **Tech stack**: Tauri v2 + Rust backend (reuse Windows `codexbar` crate where possible) + webkitgtk-rendered frontend (React/TS to match Win-CodexBar, or Svelte if friction with React-on-Linux Tauri build) — locks us in to webkitgtk 6.0+ which means Ubuntu 22.04 minimum
- **Tray**: StatusNotifierItem via DBus is the primary; AppIndicator + libayatana-appindicator3 as fallback for GNOME; degrade gracefully to a menu-less window if neither is available
- **Secrets**: Secret Service API via `secret-service` Rust crate (libsecret-backed) — no plaintext API keys on disk by default; offer encrypted-file fallback if no Secret Service is running (e.g., headless sessions)
- **Distribution**: `.deb` and AppImage as v1 targets; Flatpak is v2 (sandboxing + portal complexity)
- **Performance**: tray icon + background poller must idle at <50 MB RSS and <0.5% CPU when no popover is open
- **Timeline**: aggressive — user has asked for PRD → dev → testing → prod; expect roadmap to fit a 4–6 week build
- **No paid CI**: dev/test must run on free GitHub Actions; AppImage and `.deb` build must be CI-friendly (no codesigning hardware)
- **Security**: every secret path through Secret Service or encrypted-at-rest; no telemetry by default; opt-in crash reporting only
<!-- GSD:project-end -->

<!-- GSD:stack-start source:research/STACK.md -->
## Technology Stack

## Stack Recommendation
| Layer | Choice | Version | Confidence | Why |
|-------|--------|---------|------------|-----|
| Desktop framework | **Tauri** v2 | 2.x (latest stable) | HIGH | Windows port already on Tauri v2; reuses `codexbar` Rust crate verbatim; webkitgtk-rendered WebView on Linux; ~10–20 MB binaries vs ~150 MB Electron |
| Backend language | **Rust** 1.79+ | stable | HIGH | Reuse Win-CodexBar's `rust/` crate (41 provider impls + `Provider` trait + clap CLI) |
| Frontend stack | **React 18 + TypeScript + Vite** | match Win-CodexBar | HIGH | Identical frontend codebase between Win + Linux; webkitgtk supports React fine |
| UI styling | **Tailwind 3.x** + system theme detection | match Win-CodexBar | MEDIUM | Already in Windows port; libadwaita-style accents via CSS variables responding to `prefers-color-scheme` |
| Async runtime | **tokio** | 1.x with `rt-multi-thread`, `macros`, `time`, `signal` | HIGH | Required by reqwest + Tauri's `async_runtime` |
| HTTP client | **reqwest** with `rustls-tls` | 0.12.x | HIGH | rustls avoids OpenSSL ABI drift across Ubuntu/Fedora/Arch |
| Tray (primary) | **`tauri::tray::TrayIconBuilder`** | bundled with Tauri 2 | HIGH | Talks StatusNotifierItem on Linux; works on KDE Plasma, MATE, Cinnamon |
| Tray (GNOME fallback) | **libayatana-appindicator3** (system pkg) | 0.5+ | HIGH | Required for GNOME 45+ via AppIndicator extension; tauri-bundler auto-detects |
| Global hotkey | **tauri-plugin-global-shortcut** | 2.x | MEDIUM | Works on X11 reliably; degraded on Wayland (no portal yet) — make hotkey optional |
| Single-instance | **tauri-plugin-single-instance** | 2.x | HIGH | DBus-name lock on Linux; second-launch focuses popover |
| Autostart | **tauri-plugin-autostart** | 2.x | HIGH | Writes XDG `.desktop` to `~/.config/autostart/` |
| Notifications | **tauri-plugin-notification** | 2.x | HIGH | DBus → org.freedesktop.Notifications |
| Window state | **tauri-plugin-window-state** | 2.x | MEDIUM | Restores popover/floating-bar geometry |
| Secret storage | **`oo7`** | 0.4.x | HIGH | Secret Service DBus client with encrypted-file fallback when no keyring daemon — exactly the dual mode PROJECT.md requires |
| XDG paths | **`directories`** | 5.x | HIGH | `ProjectDirs::config_dir()` / `cache_dir()` / `state_dir()` |
| Config serialization | **serde** + **serde_json** | 1.x | HIGH | Match Win-CodexBar |
| Logging | **tracing** + **tracing-subscriber** | 0.1.x / 0.3.x | HIGH | Match Win-CodexBar; emit JSON to `XDG_STATE_HOME/codexbar/logs/` |
| CLI parser | **clap** with `derive` | 4.x | HIGH | Inherited from Win-CodexBar `codexbar` CLI |
| PTY (CLI provider runners) | **portable-pty** | 0.8.x | HIGH | Cross-platform PTY for `codex`/`claude`/`gemini` interactive sessions |
| Errors | **thiserror** | 2.x | HIGH | Used by Win-CodexBar |
| Tests | **tokio-test** + **mockito** | latest | HIGH | mockito for stubbing 41 provider HTTP endpoints |
| Property-based | **proptest** | 1.x | MEDIUM | Provider parser robustness |
## Build & Distribution Toolchain
| Stage | Tool | Notes |
|-------|------|-------|
| Bundle | `cargo tauri build --target x86_64-unknown-linux-gnu` | Tauri-bundler emits `.deb` + AppImage out of the box |
| AppImage backend | linuxdeploy (vendored by tauri-bundler) | No manual setup needed |
| deb backend | `cargo-deb` | Tauri-bundler invokes it |
| arm64 cross-build | Cross via `cross` crate or native arm64 runner | Phase-2 task; Ubuntu LTS aarch64 has growing share |
| CI | GitHub Actions `ubuntu-22.04` runner | webkitgtk 6.0 floor; matches Ubuntu 22.04 LTS user baseline |
| Glibc floor | 2.35 (Ubuntu 22.04) | Don't compile on newer Ubuntu without explicit `--target` |
| Repro builds | `SOURCE_DATE_EPOCH` + locked Cargo.lock | Deferred to v2 |
| Signing | gpg-sign `.deb` + `.AppImage` SHA-256 sidecars | No paid codesigning required |
| Auto-update | Tauri's built-in updater (signed manifest) | Pulls from GitHub Releases on a custom apt repo for `.deb` |
## What NOT to Use
| Avoid | Why |
|-------|-----|
| Electron | 150 MB+ baseline; loses the "lightweight tray app" promise |
| Native GTK4 + gtk-rs only | Would force rewriting all Windows React UI; doubles maintenance |
| Snap packaging for v1 | `system-tray` interface flaky under confinement; global-shortcut blocked; Wayland tray broken under strict snap |
| OpenSSL (`native-tls`) | ABI drift across distros — pin rustls instead |
| `keyring-rs` | Less mature Secret Service support than `oo7`; missing encrypted-file fallback |
| `appindicator-rs` (raw) | Tauri's tray bridge already wraps it — direct use is duplicate code |
| `xdotool`/`wmctrl` for hotkeys | X11-only; Wayland breaks silently |
| `eframe`/`egui`/`winit` | Already over-pulled by `codexbar` crate; conflicts with Tauri unless feature-gated off (see ARCHITECTURE.md Phase 0) |
| `directories-next` | Superseded by `directories` 5.x |
## Wayland vs X11 Implications
| Feature | X11 | Wayland (GNOME 45+) | Wayland (KDE Plasma 6) |
|---------|-----|---------------------|------------------------|
| Tray icon | StatusNotifierItem ✓ | needs AppIndicator extension | native ✓ |
| Global hotkey | works ✓ | **no portal yet** — degrade gracefully | Plasma supports KGlobalAccel — partial |
| Always-on-top floating bar | window hints ✓ | KDE-only KWayland; GNOME refuses | KWayland ✓ |
| Click-through window | XShape ✓ | not supported under most compositors | not supported |
| Window geometry restore | works ✓ | clients can request only — compositor decides | works ✓ |
## Confidence Notes
- **HIGH** picks are read from Win-CodexBar's actual `Cargo.toml` or verified via current crate listings.
- **MEDIUM** picks involve Linux-DE behavior that varies across compositor versions — must spike under real GNOME 45 + KDE Plasma 6 in Phase 2.
<!-- GSD:stack-end -->

<!-- GSD:conventions-start source:CONVENTIONS.md -->
## Conventions

Conventions not yet established. Will populate as patterns emerge during development.
<!-- GSD:conventions-end -->

<!-- GSD:architecture-start source:ARCHITECTURE.md -->
## Architecture

Architecture not yet mapped. Follow existing patterns found in the codebase.
<!-- GSD:architecture-end -->

<!-- GSD:skills-start source:skills/ -->
## Project Skills

No project skills found. Add skills to any of: `.claude/skills/`, `.agents/skills/`, `.cursor/skills/`, `.github/skills/`, or `.codex/skills/` with a `SKILL.md` index file.
<!-- GSD:skills-end -->

<!-- GSD:workflow-start source:GSD defaults -->
## GSD Workflow Enforcement

Before using Edit, Write, or other file-changing tools, start work through a GSD command so planning artifacts and execution context stay in sync.

Use these entry points:
- `/gsd-quick` for small fixes, doc updates, and ad-hoc tasks
- `/gsd-debug` for investigation and bug fixing
- `/gsd-execute-phase` for planned phase work

Do not make direct repo edits outside a GSD workflow unless the user explicitly asks to bypass it.
<!-- GSD:workflow-end -->



<!-- GSD:profile-start -->
## Developer Profile

> Profile not yet configured. Run `/gsd-profile-user` to generate your developer profile.
> This section is managed by `generate-claude-profile` -- do not edit manually.
<!-- GSD:profile-end -->
