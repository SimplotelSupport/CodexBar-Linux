# Features Research

**Domain:** Linux/Ubuntu port of CodexBar — AI coding-provider usage/quota tray dashboard
**Researched:** 2026-05-17
**Confidence:** HIGH (every feature classified against both reference repos)

---

## Feature Classification

### TABLE STAKES (v1 must-haves)

| Feature | Mac | Win | Complexity | Portability | Depends On |
|---------|-----|-----|------------|-------------|------------|
| System tray icon with dynamic glyph | ✓ | ✓ | MED | Clean (Tauri tray) | Tauri shell |
| Per-provider usage meter (% used, reset countdown) | ✓ | ✓ | MED | Clean (reuse Rust core) | Provider plugins, poller |
| Click-tray popover with provider grid | ✓ | ✓ | LOW | Clean | Tauri shell, popover positioning |
| Background poller (configurable cadence: manual/1m/2m/5m/15m) | ✓ | ✓ | LOW | Clean | tokio runtime |
| Settings window for credential management | ✓ | ✓ | MED | Clean | Secret store |
| API-key credential entry per provider | ✓ | ✓ | LOW | Clean | Secret store |
| Codex provider plugin (HTTP + JSONL cost) | ✓ | ✓ | LOW | Clean | Provider trait |
| Claude provider plugin | ✓ | ✓ | LOW | Clean | Provider trait |
| Gemini provider plugin | ✓ | ✓ | LOW | Clean | Provider trait |
| Copilot provider plugin | ✓ | ✓ | LOW | Clean | Provider trait |
| OpenRouter / OpenAI / Anthropic API provider plugins | ✓ | ✓ | LOW | Clean | Provider trait |
| Local cost scan from `~/.codex/`, `~/.claude/` JSONL logs | ✓ | ✓ | MED | Clean (paths differ) | XDG paths |
| Secret Service storage for API tokens | — | DPAPI eq. | MED | **Linux-specific design** | `oo7` |
| XDG autostart `.desktop` toggle | — | Registry eq. | LOW | **Linux-specific** | autostart plugin |
| Single-instance lock (second launch focuses popover) | ✓ | ✓ | LOW | Clean (DBus) | single-instance plugin |
| `codexbar usage -p <provider>` CLI | ✓ | ✓ | LOW | Clean (already in `codexbar` crate) | CLI subcommand |
| `codexbar cost -p <provider>` CLI | ✓ | ✓ | LOW | Clean | CLI subcommand |
| `.deb` installer for Ubuntu/Debian | — | MSI eq. | MED | **Linux-specific** | cargo-tauri/cargo-deb |
| AppImage for portable install | — | portable .exe eq. | MED | **Linux-specific** | linuxdeploy |
| GNOME + KDE Plasma compat (light + dark theme follow) | partial | partial | MED | **Linux-specific** | CSS prefers-color-scheme |
| Crash-free degradation when no tray host available | n/a | n/a | LOW | **Linux-specific** | feature detection |

### DIFFERENTIATORS (v1 nice-to-haves, v2 if cut)

| Feature | Mac | Win | Complexity | Portability | Notes |
|---------|-----|-----|------------|-------------|-------|
| Floating always-on-top usage bar | ✓ | ✓ | HIGH | Wayland-fragile | Works on X11; KDE Wayland partial; GNOME Wayland not supported — degrade with toast |
| Global hotkey to toggle popover | ✓ | ✓ | MED | Wayland-fragile | X11 works; Wayland needs xdg-desktop-portal (no GlobalShortcut portal yet) — make optional |
| Provider quota notification (toast on 80% / 100%) | ✓ | ✓ | LOW | Clean | notification plugin |
| Auto-update via Tauri updater + signed manifest | ✓ | ✓ | MED | Clean | updater plugin |
| Multi-provider "merge icons" mode (composite glyph) | ✓ | partial | MED | Clean | Tauri tray icon rebuild |
| Local-cost rolling 30-day view in popover | ✓ | ✓ | MED | Clean | Reuse Rust cost_scanner |
| Refresh-now action (manual poll) | ✓ | ✓ | LOW | Clean | IPC command |
| Per-provider enabled/disabled toggle | ✓ | ✓ | LOW | Clean | Settings store |
| Theme accent picker | partial | partial | LOW | Clean | CSS variable |
| Cursor / Cline / Aider / Continue providers (15+ additional) | ✓ | ✓ | LOW each | Clean (each is a plugin) | Many already in `codexbar` crate |
| First-run wizard | partial | partial | MED | Clean | Frontend onboarding |
| Diagnostics / "copy support bundle" | ✓ | ✓ | LOW | Clean | tracing log dump |

### DEFER TO v2

| Feature | Reason to defer |
|---------|-----------------|
| OAuth device flow per provider | Each provider differs; v1 ships API-key paste path |
| Browser cookie auto-import (Chrome/Firefox) | macOS uses Keychain, Windows DPAPI; Linux has no unified equivalent — manual cookie paste UI is enough for v1 |
| Flatpak distribution | Portal complexity (Secrets portal, Tray portal, Notification portal) — apt + AppImage cover most users |
| arm64 builds | x86_64 first; arm64 once amd64 is stable |
| Custom apt repo with GPG-signed pool | GitHub Releases `.deb` direct download is fine for v1 |
| WSLg-specific tweaks | WSL users can run the Windows port |
| Per-provider rate-limit-aware backoff | All-providers static cadence is OK; rate-limit handling iterates after telemetry |
| Telemetry / crash reporting | Privacy-first default; revisit only if support burden demands it |
| i18n | English first; translations after community traction |

### ANTI-FEATURES (explicitly NOT building)

| Feature | Why we refuse |
|---------|---------------|
| Snap package | `system-tray` interface degraded; global-shortcut blocked under confinement; Wayland tray broken under strict snap — bad UX |
| 32-bit i386 build | Ubuntu LTS dropped i386 desktop; serves nobody |
| Custom theming engine | System theme inheritance is the right Linux behavior; over-skinning fights every DE |
| Cross-DE tray emulation (XEmbed) | XEmbed is obsolete; modern DEs ignore it; we use SNI/AppIndicator only |
| WidgetKit-equivalent desktop widgets | No portable Linux mechanism; GNOME extensions are not a stable surface |
| Sparkle-style delta auto-update | Tauri updater + AppImage update is idiomatic; differential updates are over-engineering for <20 MB binaries |
| Persistent telemetry-by-default | Privacy-first; only opt-in crash reports later |
| Local model proxying / man-in-the-middle | Out of scope — we report usage, we don't proxy traffic |

## Dependency Graph (table-stakes only)

```
                    ┌── Tauri shell ───────────────────────────┐
                    │                                          │
                    ▼                                          ▼
            single-instance lock                     tray icon (Tauri tray)
                    │                                          │
                    ▼                                          ▼
            popover window  ◄────────  IPC commands  ────────► settings window
                    │                                          │
                    ▼                                          ▼
            UsageStore (Arc<RwLock>)                     Secret Service (oo7)
                    ▲                                          │
                    │                                          │
            background poller (tokio task)                     │
                    ▲                                          │
                    │                                          │
            Provider trait + 41 plugin impls  ◄───────────────┘
                    ▲
                    │
            reqwest / portable-pty / cost_scanner
```

## Linux-Only Design Considerations

1. **Tray host detection** at startup — probe StatusNotifierItem via DBus; if absent, probe `libayatana-appindicator`; if both absent, surface a one-time toast "No system tray detected — popover is keyboard-accessible only" and proceed.
2. **XDG paths everywhere** — `XDG_CONFIG_HOME/codexbar/config.toml`, `XDG_CACHE_HOME/codexbar/usage_snapshots/`, `XDG_STATE_HOME/codexbar/logs/`.
3. **Autostart** writes `~/.config/autostart/codexbar.desktop` with proper `X-GNOME-Autostart-enabled=true` and `OnlyShowIn=` left empty for portability.
4. **Theme follow** uses `gtk-application-prefer-dark-theme` for GNOME and `qt5ct`/`kde-config-watcher` for KDE — most of this is automatic via webkitgtk's `prefers-color-scheme`.
5. **JSONL cost scan paths** — Linux uses `~/.codex/sessions/`, `~/.claude/conversations/` (not `~/Library/...`); detect and warn if not present.
