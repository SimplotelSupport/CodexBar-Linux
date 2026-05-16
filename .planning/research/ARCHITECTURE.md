# Architecture Research

**Domain:** Linux system-tray desktop app (Tauri v2 + Rust, AI provider usage monitor)
**Researched:** 2026-05-16
**Confidence:** HIGH (Windows port source read directly; Tauri v2 docs verified via search)

---

## Standard Architecture

### System Overview

```
┌──────────────────────────────────────────────────────────────────────────┐
│  UI LAYER  (webkitgtk WebView — React/TS, same frontend as Windows port) │
│  ┌──────────────┐  ┌──────────────────┐  ┌─────────────────────────────┐ │
│  │ Popover      │  │ Floating Bar     │  │ Settings Window             │ │
│  │ (provider    │  │ (always-on-top   │  │ (credentials, poll cadence, │ │
│  │  grid, usage │  │  compact strip)  │  │  theme, autostart)          │ │
│  │  bars, reset │  └──────────────────┘  └─────────────────────────────┘ │
│  │  countdown)  │                                                         │
│  └──────────────┘                                                         │
├──────────────────────────────────────────────────────────────────────────┤
│  TAURI IPC BRIDGE  (#[tauri::command] async fn, tauri::async_runtime)    │
│  ┌──────────┐  ┌──────────────┐  ┌────────────────┐  ┌────────────────┐ │
│  │ commands/│  │ tray_bridge  │  │ floatbar/      │  │ shell/         │ │
│  │ tokens   │  │ (TrayIcon-   │  │ window.rs      │  │ settings_win,  │ │
│  │ chart    │  │  Builder,    │  │                │  │ position)      │ │
│  │ diag     │  │  menu, icon) │  │                │  │                │ │
│  │ updater  │  └──────────────┘  └────────────────┘  └────────────────┘ │
├──────────────────────────────────────────────────────────────────────────┤
│  RUST CORE  (crate: `codexbar`, path: rust/ — forked from Win-CodexBar) │
│  ┌──────────────┐  ┌───────────┐  ┌──────────────┐  ┌────────────────┐  │
│  │ core/        │  │ providers/│  │ settings.rs  │  │ secure_file.rs │  │
│  │  Provider    │  │  41 impls │  │ (config,     │  │ (encrypted     │  │
│  │  trait       │  │  factory  │  │  poll cadence│  │  fallback)     │  │
│  │  UsageSnapshot│ │  mod.rs)  │  │  enabled set)│  │                │  │
│  │  FetchContext│  └───────────┘  └──────────────┘  └────────────────┘  │
│  │  ProviderError│                                                        │
│  └──────────────┘                                                         │
│  ┌──────────────┐  ┌───────────┐  ┌──────────────┐  ┌────────────────┐  │
│  │ cost_scanner │  │ cli/      │  │ logging/     │  │ status/        │  │
│  │ (JSONL scan) │  │ (clap CLI │  │ (tracing)    │  │ indicators.rs  │  │
│  │              │  │  subcommands)│              │  │                │  │
│  └──────────────┘  └───────────┘  └──────────────┘  └────────────────┘  │
├──────────────────────────────────────────────────────────────────────────┤
│  LINUX PLATFORM LAYER  (new code — Linux port additions only)            │
│  ┌──────────────────┐  ┌────────────────┐  ┌──────────────────────────┐  │
│  │ tray_linux/      │  │ secret store   │  │ single-instance          │  │
│  │ SNI + AppInd.    │  │ (oo7 crate:    │  │ (tauri-plugin-single-    │  │
│  │ fallback detect  │  │  Secret Service│  │  instance — DBus on      │  │
│  │ GNOME/KDE/none   │  │  or enc. file) │  │  Linux)                  │  │
│  └──────────────────┘  └────────────────┘  └──────────────────────────┘  │
├──────────────────────────────────────────────────────────────────────────┤
│  EXTERNAL SYSTEMS                                                         │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐                    │
│  │ Provider APIs│  │ CLI tools    │  │ JSONL logs   │                    │
│  │ (reqwest,    │  │ (codex,      │  │ (~/.codex/,  │                    │
│  │  41 HTTP     │  │  claude,     │  │  ~/.claude/) │                    │
│  │  endpoints)  │  │  portable-pty│  │              │                    │
│  └──────────────┘  └──────────────┘  └──────────────┘                    │
└──────────────────────────────────────────────────────────────────────────┘
```

---

## Cargo Workspace Layout

The Windows port is a two-member Cargo workspace (`rust/` + `apps/desktop-tauri/src-tauri`). The Linux port mirrors this structure with a new Linux-specific Tauri crate.

### Critical Decision: How to Consume the `codexbar` Crate

**This is a Phase 0 decision that blocks everything else.** The `codexbar` crate (in `rust/`) is the shared Rust backend — but in its current form in Win-CodexBar, it pulls GUI dependencies unconditionally in `[dependencies]` (not behind feature flags):

```toml
eframe = "0.30"        # full egui GUI framework + wgpu renderer
egui = "0.30"
winit = "0.30"         # Win-CodexBar's own event loop
tray-icon = "0.19"     # Win-CodexBar's standalone tray (not Tauri's)
muda = "0.15"
global-hotkey = "0.7.0"
```

These conflict with Tauri v2's own copies of `tray-icon` and `winit` on Linux, causing version-skew and double-linking issues. `eframe`/`wgpu` also pull heavy GPU linkage unused in the Tauri build.

**Three options — pick one in Phase 0:**

| Option | Work | Risk |
|--------|------|------|
| **(A) Fork + add feature flags** — move GUI deps behind `[features] gui-egui = [...]`, default off | One upstream PR; Linux port depends on fork until merged | Upstream may be slow to accept; fork divergence |
| **(B) Maintain a stripped Linux fork** — vendor `rust/` at a commit, strip GUI deps, sync manually on upstream changes | Low initial effort | Merge debt grows with each upstream release |
| **(C) Accept the bloat, version-pin everything** | Fastest to first light | `tray-icon` version conflict almost certain; CI on headless Linux will need display server for winit |

**Recommendation: Option A.** Fork Win-CodexBar's `rust/` crate, add `gui-egui` feature (default off), move `eframe/egui/winit/tray-icon/muda/global-hotkey` under `[features]`. The Linux Tauri crate depends on `codexbar` without that feature. Open upstream PR in parallel. The Linux port's repo hosts the fork at `rust/` using a `[patch.crates-io]` or path dep.

Until this decision is made and executed, "Pattern 1: Headless Core + Platform Shell" is the **target state**. Phase 1 begins by making it the actual state.

### Workspace Structure (target state after Phase 0)

```
CodexBar-Linux/                     ← git root
├── Cargo.toml                      ← workspace root
│     members = ["rust", "apps/desktop-linux/src-tauri"]
│     resolver = "3"
│
├── rust/                           ← crate name: `codexbar` (forked from Win-CodexBar)
│   ├── Cargo.toml                  ← no Tauri deps; GUI deps behind `gui-egui` feature (off)
│   ├── src/
│   │   ├── lib.rs                  ← pub mod core, providers, settings, cli, ...
│   │   ├── core/
│   │   │   ├── provider.rs         ← Provider trait, FetchContext, ProviderError
│   │   │   ├── usage_snapshot.rs   ← UsageSnapshot, RateWindow, NamedRateWindow
│   │   │   ├── credentials.rs      ← credential model
│   │   │   ├── jsonl_scanner.rs    ← local log cost scanning
│   │   │   └── ...
│   │   ├── providers/
│   │   │   ├── mod.rs              ← 41 Provider impls, factory/mod.rs
│   │   │   ├── claude/mod.rs
│   │   │   ├── codex/mod.rs
│   │   │   └── ...
│   │   ├── settings.rs             ← AppSettings, poll cadence, enabled set
│   │   ├── secure_file.rs          ← encrypted-file fallback (no keyring dep)
│   │   ├── cli/                    ← clap CLI subcommands (usage, cost)
│   │   └── ...
│   └── tests/
│       └── providers/              ← integration tests with mockito
│
└── apps/
    └── desktop-linux/
        ├── src/                    ← React/TS frontend (ported from Win-CodexBar)
        │   ├── App.tsx
        │   ├── components/
        │   └── ...
        └── src-tauri/             ← crate name: `codexbar-desktop-linux`
            ├── Cargo.toml         ← deps: codexbar (path ../../../rust), tauri v2
            ├── tauri.conf.json
            ├── capabilities/
            └── src/
                ├── main.rs
                ├── lib.rs
                ├── commands/       ← #[tauri::command] handlers
                │   ├── mod.rs
                │   ├── tokens.rs
                │   ├── chart.rs
                │   └── diagnostics.rs
                ├── tray_bridge.rs  ← TrayIconBuilder, menu, icon, SNI fallback logic
                ├── floatbar/       ← floating bar window management
                │   ├── mod.rs
                │   └── window.rs
                ├── shell/          ← window geometry, settings window, positioning
                │   ├── mod.rs
                │   ├── settings_window.rs
                │   ├── position.rs
                │   └── geometry.rs
                ├── secret_store.rs ← oo7 wrapper (Secret Service + enc. file)
                ├── shortcut_bridge.rs ← global-shortcut (best-effort Wayland)
                ├── events.rs       ← event type definitions for frontend emit
                ├── state.rs        ← AppState (Mutex<> wrappers for shared state)
                └── surface.rs      ← SurfaceMode, popover vs floatbar transitions
```

**Invariant (target state):** `rust/` has zero Tauri imports. `codexbar-desktop-linux` is the only crate that imports `tauri`. This boundary is what lets `cargo test -p codexbar` run in headless CI without a display server.

**Note on `global-hotkey`:** `rust/Cargo.toml` has `global-hotkey = "0.7.0"` as a direct dep (used by the standalone egui app). The Tauri Linux port uses `tauri-plugin-global-shortcut` instead. Strip `global-hotkey` from `rust/Cargo.toml` when feature-gating the egui mode.

---

## Component Boundaries

| Component | Crate | Responsibility | Communicates With |
|-----------|-------|---------------|-------------------|
| `codexbar::core::Provider` (trait) | `rust/` | Contract for all 41 providers | Nothing — it's a trait |
| `codexbar::providers::*` (41 impls) | `rust/` | HTTP/CLI fetch, parse, return `UsageSnapshot` | `reqwest`, `portable-pty`, system CLIs |
| `codexbar::settings` | `rust/` | Deserialize/serialize `AppSettings` from/to TOML | XDG config path |
| `codexbar::cost_scanner` | `rust/` | Walk `~/.codex/` and `~/.claude/` JSONL, sum tokens | Local filesystem |
| `codexbar::cli` | `rust/` | `codexbar usage`, `codexbar cost` subcommands | providers, settings, stdout |
| `TrayBridge` | `codexbar-desktop-linux` | Build tray icon, detect SNI vs AppIndicator vs none, route click events | Tauri runtime, frontend via emit |
| `PopoverWindow` | `codexbar-desktop-linux` | Create/show/hide the WebView popover on tray click | Tauri window API, frontend |
| `FloatBar` | `codexbar-desktop-linux` | Always-on-top strip window, toggle via command | Tauri window API, frontend |
| `SettingsWindow` | `codexbar-desktop-linux` | Manage settings WebView, save to config | `codexbar::settings`, `SecretStore` |
| `SecretStore` | `codexbar-desktop-linux` | `oo7` wrapper: Secret Service first, encrypted file fallback | libsecret DBus or `$XDG_DATA_HOME/codexbar/secrets.enc` |
| `ShortcutBridge` | `codexbar-desktop-linux` | `tauri-plugin-global-shortcut` registration | Tauri, TrayBridge |
| `commands::*` | `codexbar-desktop-linux` | IPC: `get_usage`, `refresh_now`, `set_enabled`, `get_settings` | `AppState`, `codexbar::providers::*` |
| `AppState` | `codexbar-desktop-linux` | `Mutex<HashMap<ProviderId, UsageSnapshot>>` + poll task handle | Tauri managed state |

---

## Data Flow

### Refresh-Now Tick: Full Path

```
USER CLICKS "Refresh" in Popover WebView
    │
    ▼  (JS invoke)
tauri::command::refresh_now(app: AppHandle)
    │  tauri::async_runtime::spawn()
    ▼
ProviderPoller::poll_all(providers, ctx)
    │  for each enabled Provider:
    │  tauri::async_runtime::spawn() per provider
    ▼
Provider::fetch_usage(&FetchContext) → async fn
    │
    ├─[HTTP path]─► reqwest client → provider API → parse JSON
    │
    └─[CLI path]──► spawn subprocess via portable-pty
                    → read stdout/stderr
                    → parse JSONL / text output
    │
    ▼
Result<ProviderFetchResult, ProviderError>
    │
    ▼
AppState: Mutex<HashMap<ProviderId, UsageSnapshot>>.lock().insert(...)
    │
    ▼
app.emit("usage-updated", &payload)  ← Tauri event to all windows
    │
    ▼
WebView React: useEffect listens → re-render provider grid, usage bars,
               countdown timers, tray icon badge update via command
```

### Poll Scheduler (background)

```
setup() hook in lib.rs
    │
    ▼
tauri::async_runtime::spawn(async {
    let mut interval = tokio::time::interval(poll_cadence);
    loop {
        interval.tick().await;
        poll_all(...).await;
    }
})
```

### Settings Save

```
WebView: user edits settings form → JS invoke("save_settings", payload)
    │
    ▼
commands::save_settings(settings: AppSettings)
    │
    ├── write $XDG_CONFIG_HOME/codexbar/config.toml  (codexbar::settings)
    └── for each secret: SecretStore::store(provider_id, credential)
              │
              ├─[keyring avail]─► oo7 → DBus → GNOME Keyring / KWallet
              └─[no keyring]────► oo7 → XChaCha20-Poly1305 enc. file
                                        at $XDG_DATA_HOME/codexbar/secrets.enc
```

### Second-Instance Focus

```
Second launch of codexbar binary
    │
    ▼
tauri-plugin-single-instance (registered FIRST in plugin chain)
    │  DBus: try to own "com.codexbar.app"
    │  name already owned → send args to first instance via DBus
    ▼
First instance: plugin emits "single-instance" event in Rust
    │
    ▼
handler: show/focus popover window
```

---

## Threading Model

```
┌────────────────────────────────────────────────────────────────┐
│ MAIN THREAD (GTK event loop, required by webkitgtk)            │
│  - Tauri app.run()                                             │
│  - TrayIconBuilder events                                      │
│  - Window create/show/hide                                     │
│  - tauri-plugin-single-instance DBus listener                  │
│  - tauri-plugin-global-shortcut registration                   │
└────────────────────────────────────────────────────────────────┘
         │ tauri::async_runtime::spawn()
         ▼
┌────────────────────────────────────────────────────────────────┐
│ TOKIO RUNTIME (multi-thread, managed by Tauri)                 │
│  - ProviderPoller interval loop                                │
│  - per-provider fetch_usage() tasks (concurrent)              │
│  - reqwest HTTP tasks                                          │
│  - IPC command handlers (async #[tauri::command])              │
│         │ spawn_blocking()                                     │
│         ▼                                                      │
│  ┌─────────────────────────────────────────────────────────┐  │
│  │ BLOCKING POOL (tokio::task::spawn_blocking)              │  │
│  │  - JSONL log scan (cost_scanner, filesystem walk)        │  │
│  │  - portable-pty subprocess reads (claude, codex CLIs)   │  │
│  │  - settings TOML file I/O                                │  │
│  └─────────────────────────────────────────────────────────┘  │
└────────────────────────────────────────────────────────────────┘
```

**Rule:** Never use `tokio::spawn` directly inside Tauri event listeners or window handlers — use `tauri::async_runtime::spawn` to avoid "no reactor running" panics (upstream issue tauri-apps/tauri#10289).

---

## Provider Plugin Contract

The `Provider` trait is **already defined in the Windows port** and must be reused verbatim. Do not create a new trait.

```rust
// rust/src/core/provider.rs — EXISTING, do not change

#[async_trait]
pub trait Provider: Send + Sync {
    fn id(&self) -> ProviderId;
    fn metadata(&self) -> &ProviderMetadata;
    async fn fetch_usage(&self, ctx: &FetchContext) -> Result<ProviderFetchResult, ProviderError>;

    // Default implementations — override in Linux-specific providers if needed:
    fn available_sources(&self) -> Vec<SourceMode> { vec![SourceMode::Auto] }
    fn supports_oauth(&self) -> bool { false }
    fn supports_web(&self) -> bool { false }   // cookie-based
    fn supports_cli(&self) -> bool { false }   // PTY spawn
    fn detect_version(&self) -> Option<String> { None }
}
```

Key types already defined:
- `FetchContext` — source mode, api_key, timeout, manual_cookie_header
- `ProviderError` — NotInstalled, AuthRequired, Network, Timeout, Parse, NoCookies, Other
- `UsageSnapshot` — primary/secondary/model_specific/tertiary `RateWindow`, updated_at, account metadata
- `SourceMode` — Auto | OAuth | Web | Cli

**Adding a Linux-only provider:** implement `Provider`, add variant to `ProviderId` enum, add to `providers/mod.rs` re-exports, add to `factory/mod.rs`. All existing tests and CLI code continue working.

**Mock pattern for tests:** implement `Provider` for a test struct that returns a fixed `UsageSnapshot`. No mockall needed — the trait is simple enough for manual mocks. Use `mockito` for HTTP-level mocks (already in `[dev-dependencies]`).

```rust
// Example test stub
struct MockClaude { snapshot: UsageSnapshot }

#[async_trait]
impl Provider for MockClaude {
    fn id(&self) -> ProviderId { ProviderId::Claude }
    fn metadata(&self) -> &ProviderMetadata { &CLAUDE_METADATA }
    async fn fetch_usage(&self, _ctx: &FetchContext) -> Result<ProviderFetchResult, ProviderError> {
        Ok(ProviderFetchResult { snapshot: self.snapshot.clone(), diagnostics: vec![] })
    }
}
```

**Rate-limit awareness:** `ProviderError::Timeout` and `ProviderError::Network` are returned; callers in the poller back off per-provider. No shared rate-limit registry needed in v1 — each provider controls its own retry logic internally.

---

## Persistence Layout (XDG-Compliant)

| Data | Path | Crate | Notes |
|------|------|-------|-------|
| User config | `$XDG_CONFIG_HOME/codexbar/config.toml` | `codexbar::settings` | Poll cadence, enabled providers, theme, autostart |
| Credential secrets | Secret Service (GNOME Keyring / KWallet) | `oo7` | Primary; keyed by `"codexbar/{provider_id}"` |
| Credential fallback | `$XDG_DATA_HOME/codexbar/secrets.enc` | `oo7` enc. file | Used when no Secret Service daemon running |
| Usage snapshots cache | `$XDG_CACHE_HOME/codexbar/snapshots/{provider}.json` | `codexbar-desktop-linux` | Written after each successful fetch; read on startup to avoid blank state |
| Structured logs | `$XDG_STATE_HOME/codexbar/logs/codexbar.log` | `tracing-subscriber` | Rotated at 10 MB; kept 3 generations |
| Autostart desktop entry | `$XDG_CONFIG_HOME/autostart/codexbar.desktop` | `codexbar-desktop-linux` | Written/removed by Settings toggle |
| Cost scanner JSONL | `~/.codex/` and `~/.claude/` (read-only) | `codexbar::cost_scanner` | Not written by this app; scanned up to 30 days |

`$XDG_CONFIG_HOME` defaults to `~/.config` if unset.
`$XDG_CACHE_HOME` defaults to `~/.cache`.
`$XDG_STATE_HOME` defaults to `~/.local/state`.
`$XDG_DATA_HOME` defaults to `~/.local/share`.

Use the `directories` crate (v6, already in `rust/Cargo.toml`) for all path resolution — it respects XDG env vars correctly. Use `ProjectDirs::from("", "", "codexbar").state_dir()` for the log path (`$XDG_STATE_HOME/codexbar`). Verified: `ProjectDirs::state_dir()` exists in directories v5+ and returns `~/.local/state/<app>` on Linux.

---

## Single-Instance Enforcement

Use `tauri-plugin-single-instance` (already used in the Windows port's `Cargo.toml`).

On Linux, this plugin acquires a DBus name `com.codexbar.app` at startup. A second launch detects the name is already owned, sends its argv over DBus to the first instance, and exits. The first instance receives a `"single-instance"` Rust event and responds by showing/focusing the popover window.

**Must be registered first** in the plugin chain (before any other plugin) for reliable operation.

---

## Linux Tray: DE Detection and Fallback

**Confidence: MEDIUM** — GNOME 45 AppIndicator regression is known but behavior varies by distro patching. Validate in Phase 2 spike.

```
startup: detect tray protocol
    │
    ├─[SNI available via DBus / KDE Plasma]──► TrayIconBuilder (Tauri built-in)
    │   uses tray-icon crate which speaks StatusNotifierItem
    │
    ├─[GNOME Shell + libayatana-appindicator3 installed]──► AppIndicator3 path
    │   Tauri's tray-icon falls back automatically on GNOME if appindicator is present
    │
    └─[no tray protocol detected]──► degraded mode
          - Show a regular always-on-top window at startup
          - Register global hotkey (best-effort)
          - `codexbar show` CLI command focuses window
          - Log warning: tray unavailable, using fallback window
```

**Wayland hotkey:** `tauri-plugin-global-shortcut` has limited Wayland support (requires `wlr-layer-shell` or `xdg-desktop-portal` which is not universally available). Treat as best-effort. On X11 and XWayland it works reliably. Known degradation — do not block release on it.

---

## Build Order (Phase Dependencies)

```
Phase 0 — Crate strategy decision (before any code)
  [0] Decide how rust/ (codexbar crate) is consumed (see "Critical Decision" above)
      - Fork Win-CodexBar/rust/, add gui-egui feature gate (recommended)
      - Strip eframe/egui/winit/tray-icon/muda/global-hotkey from default deps
      - Confirm `cargo build -p codexbar` succeeds on Linux without display server
  VERIFY: CI headless build green; no link errors from tray-icon version collision

Phase 1 — Core foundation (no UI)
  [1] rust/ crate compiles for Linux target (after Phase 0 fork/feature-gate)
      - Add oo7 for secret store (replaces Windows `keyring` crate for Linux path)
      - Audit all 41 providers for Windows I/O assumptions (registry reads, DPAPI)
      - Add cfg(unix) alternatives where needed
  [2] codexbar CLI binary (`codexbar usage`, `codexbar cost`) works on Linux
  VERIFY: `cargo test -p codexbar` passes in CI; `codexbar usage --provider claude` runs

Phase 2 — Tauri shell (tray + popover)
  [3] apps/desktop-linux/src-tauri scaffolded, depends on rust/ crate
  [4] TrayBridge: tray icon renders on GNOME/KDE (spike both DEs)
  [5] PopoverWindow: click tray → show WebView window
  [6] Single-instance plugin wired (first in chain)
  [7] Tray-unavailable degraded mode (always-on-top fallback window)
  VERIFY: App launches, tray visible on GNOME + KDE; click opens window; second
          launch focuses first; degraded mode shows window when tray missing

Phase 3 — Provider data pipeline
  [8] AppState + ProviderPoller (tokio interval, per-provider spawn)
  [9] IPC commands: get_usage, refresh_now, set_enabled
  [10] Frontend: provider grid, usage bars, countdown timers
  [11] XDG snapshot cache (write on fetch, read on startup)
  VERIFY: Refresh populates real data; restart shows cached data within 100ms

Phase 4 — Credentials + settings
  [12] SecretStore (oo7): Secret Service primary + encrypted file fallback
  [13] SettingsWindow: enable/disable providers, set API keys, poll cadence
  [14] Autostart .desktop file generation
  VERIFY: Key stored in GNOME Keyring; survives reboot; headless enc-file fallback works

Phase 5 — Polish + distribution
  [15] FloatBar window (always-on-top strip)
  [16] Global hotkey (tauri-plugin-global-shortcut, best-effort Wayland)
  [17] GNOME libadwaita CSS + KDE Plasma theme inheritance
  [18] .deb package (tauri bundler) + AppImage
  [19] Auto-update (Tauri updater plugin or custom apt repo)
  VERIFY: deb installs clean; AppImage runs on Ubuntu 22.04 + Fedora 39
```

**DAG (what must exist before what):**

```
Phase 0: crate strategy decision
    │
    ▼
rust/ (codexbar crate, feature-gated) ──────────────────────┐
  │ no platform-specific GUI deps                            │
  ▼                                                          ▼
codexbar CLI binary                    codexbar-desktop-linux (Tauri)
  (Phase 1)                              │
                                         ├── TrayBridge (Phase 2)
                                         ├── AppState + Poller (Phase 3)
                                         ├── SecretStore (Phase 4)
                                         └── FloatBar (Phase 5)
```

---

## Architectural Patterns

### Pattern 1: Headless Core + Platform Shell (target state)

The `codexbar` crate must know nothing about Tauri, GTK, or DBus. It only knows about HTTP, PTY, JSON, and XDG paths. The Tauri crate is a thin shell that manages windows, tray, and Tauri IPC on top of the core.

**Current state:** The Windows port's `codexbar` crate has GUI deps (`eframe`, `winit`, `tray-icon`) compiled unconditionally. Phase 0 is making it actually headless via feature-gating.

**When to use:** Always. This boundary is what makes the CLI binary possible and what lets tests run without a display server.

**Trade-off:** You cannot call `app.emit()` from inside the `codexbar` crate; providers signal completion only via `Result<>` return values. The Tauri layer polls or wraps in a channel.

### Pattern 2: Per-Provider Concurrency via spawn

Each enabled provider fetch runs in its own `tauri::async_runtime::spawn` task. Failures in one do not block others. Results are written individually to `AppState`.

**When to use:** Always for the poll loop. Do not `join_all` and wait for the slowest provider.

**Trade-off:** UI can receive partial updates (some providers updated, others still stale). Frontend must handle this gracefully — stale indicator per card.

### Pattern 3: Snapshot Cache for Startup Latency

On startup, read `$XDG_CACHE_HOME/codexbar/snapshots/*.json` before the first poll completes. Popover shows stale-but-nonzero data immediately; fresh data overwrites within the first poll interval.

**When to use:** Always — prevents blank/spinner state on launch.

---

## Anti-Patterns

### Anti-Pattern 1: Provider depending on Tauri

Adding `use tauri::AppHandle` inside `rust/src/providers/*.rs` kills CLI usability and breaks the test suite. The `codexbar` crate must have zero Tauri imports.

**Do this instead:** providers return `Result<>`, callers (in the Tauri commands layer) decide what to do with it.

### Anti-Pattern 2: tokio::spawn inside Tauri event listeners

Raw `tokio::spawn` panics with "no reactor running" when called from Tauri window event listeners or menu event closures (issue tauri-apps/tauri#10289).

**Do this instead:** Use `tauri::async_runtime::spawn` exclusively inside the Tauri crate.

### Anti-Pattern 3: Blocking the main thread for provider fetches

Calling `.block_on()` or any sync HTTP in the GTK main thread causes the tray and all windows to freeze.

**Do this instead:** All provider fetches go through `tauri::async_runtime::spawn`. Filesystem scans (JSONL) go through `spawn_blocking`.

### Anti-Pattern 4: Hardcoding `~/.config` instead of XDG env vars

On NixOS and some Arch setups, `$XDG_CONFIG_HOME` is non-default. Hardcoding breaks these users.

**Do this instead:** Always resolve via the `directories` crate's `ProjectDirs::from("", "", "codexbar")`.

### Anti-Pattern 5: Registering tauri-plugin-single-instance after other plugins

The single-instance plugin must be first in the `.plugin()` chain. Registering it later means other plugins may open windows before the duplicate check fires.

### Anti-Pattern 6: Pulling `codexbar` without feature-gating the GUI deps

Adding `codexbar = { path = "../../../rust" }` in `codexbar-desktop-linux/Cargo.toml` while the crate still has `eframe`, `winit`, and `tray-icon = "0.19"` as unconditional deps will conflict with Tauri's own `tray-icon` (different version, different SNI wiring) and drag wgpu/GPU linkage into every build.

**Do this instead:** Complete Phase 0 (feature-gate GUI deps) before adding any Tauri crate dep on the `codexbar` crate.

---

## Integration Points

### External Services

| Service | Integration Pattern | Notes |
|---------|---------------------|-------|
| 41 AI provider APIs | `reqwest` async HTTP, provider-specific JSON parsing | Some use OAuth token refresh (copilot, vertexai) |
| GNOME Keyring / KWallet | `oo7` crate via DBus (Secret Service API) | Falls back to encrypted file if daemon absent |
| codex / claude CLIs | `portable-pty` subprocess spawn, stdout parse | PTY needed for CLIs that detect non-TTY and refuse output |
| Local JSONL logs | Sync filesystem walk in `spawn_blocking` | Read-only; `~/.codex/` and `~/.claude/` |
| StatusNotifierItem | Tauri built-in `tray-icon` crate (SNI protocol) | Works on KDE Plasma 5/6; GNOME needs appindicator extension |
| AppIndicator3 | `libayatana-appindicator3` system lib via Tauri | GNOME fallback; must be installed by user on GNOME 45+ |
| DBus session bus | `zbus` (used internally by oo7 + tauri-plugin-single-instance) | Required; always present on GNOME/KDE |

### Internal Boundaries

| Boundary | Communication | Notes |
|----------|---------------|-------|
| `codexbar` → `codexbar-desktop-linux` | Rust function calls, types (no IPC) | One-way: Tauri crate calls into core crate |
| Commands layer → AppState | `Mutex<>` lock, clone snapshot | Keep lock duration minimal; never hold across await |
| AppState → WebView | `app.emit("usage-updated", payload)` | JSON-serialized `UsageSnapshot` subset |
| WebView → Commands | `invoke("command_name", args)` | Standard Tauri IPC |
| Poller → Commands | `tauri::async_runtime::spawn` + `AppHandle` | Poller holds `AppHandle` clone for emit |

---

## Sources

- Win-CodexBar source (direct read): `Cargo.toml`, `rust/Cargo.toml`, `rust/src/core/provider.rs`, `rust/src/lib.rs`, `apps/desktop-tauri/src-tauri/Cargo.toml` — HIGH confidence
- Tauri v2 async_runtime docs: https://docs.rs/tauri/latest/tauri/async_runtime/index.html — HIGH confidence
- tauri-apps/tauri issue #10289 (tokio::spawn panic in window listeners) — HIGH confidence
- tauri-plugin-single-instance Linux DBus mechanism: https://v2.tauri.app/plugin/single-instance/ — HIGH confidence
- `oo7` crate (Secret Service + encrypted file fallback): https://crates.io/crates/oo7 — HIGH confidence
- `directories` crate `ProjectDirs::state_dir()` verified via Context7 docs — HIGH confidence
- GNOME 45 AppIndicator regression: community reports (WebSearch) — MEDIUM confidence, validate in Phase 2 spike

---

*Architecture research for: CodexBar Linux — Tauri v2 + Rust tray app, Linux port of Win-CodexBar*
*Researched: 2026-05-16*
