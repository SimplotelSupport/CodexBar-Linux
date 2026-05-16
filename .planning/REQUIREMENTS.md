# Requirements: CodexBar Linux

**Defined:** 2026-05-17
**Core Value:** A glanceable Linux tray indicator that tells a developer *right now* how much of their AI provider quota is left and when it resets — without leaving their editor.

## v1 Requirements

Requirements for the initial public release on Ubuntu 22.04 LTS / Debian 12 / Fedora 39+ / Arch.

### Core Backend (`codexbar-core` — selectively vendored from Win-CodexBar `rust/`)

- [ ] **CORE-01**: `codexbar-core` crate vendors only headless modules from Win-CodexBar (`core/`, `providers/{codex,claude,openai,openaiapi,openrouter}/`, `cli/`, `cost_scanner.rs`, `secure_file.rs`, `settings.rs`, `logging.rs`); GUI/Windows-specific modules (`tray/`, `host/`, `browser/`, `notifications.rs`, `sound.rs`, `wsl.rs`, `login.rs`, `updater.rs`, `shortcuts.rs`) are NOT vendored
- [ ] **CORE-02**: `codexbar-core` Cargo.toml has zero GUI/Windows dependencies; `cargo build -p codexbar-core` succeeds on `x86_64-unknown-linux-gnu` with default features (no feature flags required); `cargo tree -p codexbar-core` shows no `winit`, `eframe`, `egui`, `tray-icon`, `muda`, `global-hotkey`, `keyring`, `aes-gcm`, `winreg`, `windows` (the crate)
- [ ] **CORE-03**: `reqwest` is configured with `rustls-tls` and `default-features = false` to avoid OpenSSL ABI drift; vendored crate must not link `native-tls` or `openssl-sys`
- [ ] **CORE-04**: `codexbar-cli` crate produces a stand-alone CLI binary (`codexbar`) usable independently of the desktop shell
- [ ] **CORE-05**: All XDG paths resolved via `directories::ProjectDirs` (config, cache, state directories follow XDG Base Directory spec)
- [ ] **CORE-06**: Structured logging via `tracing` writes JSON to `$XDG_STATE_HOME/codexbar/logs/` with daily rotation and 7-day retention

### Provider Plugins (alpha = 5 providers)

- [ ] **PROV-01**: Codex provider plugin: HTTP usage query + local JSONL cost scan from `~/.codex/sessions/`
- [ ] **PROV-02**: Claude provider plugin: HTTP usage + local JSONL cost scan from `~/.claude/conversations/` (this covers the Anthropic CLI path)
- [ ] **PROV-03**: OpenAI provider plugin (CLI / API key path)
- [ ] **PROV-04**: Anthropic API provider plugin (raw API key path via `openaiapi/`-style shape)
- [ ] **PROV-05**: OpenRouter provider plugin
- [ ] **PROV-06**: Every provider plugin has a `serde::Deserialize`-typed response struct (no `serde_json::Value` parsing in the hot path)
- [ ] **PROV-07**: Every provider plugin has at least one mockito-backed fixture test that fails when upstream response shape changes
- [ ] **PROV-08**: Background poller respects per-provider `min_interval` and 429 `Retry-After` headers; exponential backoff with jitter on consecutive failures
- [ ] **PROV-09**: Circuit breaker pauses a provider after 5 sequential failures until next manual refresh

### Tray UI (Tauri v2 + webkitgtk WebView)

- [ ] **TRAY-01**: System tray icon appears on KDE Plasma 5/6, XFCE, Cinnamon, MATE via StatusNotifierItem (DBus)
- [ ] **TRAY-02**: System tray icon appears on GNOME 45+ via libayatana-appindicator3 fallback
- [ ] **TRAY-03**: Tray-host detection at startup; if no tray host found, app surfaces a non-blocking toast and continues with keyboard-accessible popover
- [ ] **TRAY-04**: Tray icon glyph updates in real time when poller delivers new usage data (no need to open popover to refresh)
- [ ] **TRAY-05**: Left-click on tray icon opens popover anchored to tray bounds (or cursor on Wayland where bounds are unavailable)
- [ ] **TRAY-06**: Right-click on tray icon shows menu: Refresh now / Settings / Quit
- [ ] **TRAY-07**: Popover renders a provider grid showing per-provider usage % and reset countdown
- [ ] **TRAY-08**: Popover usage bars update via Tauri IPC events without re-rendering the whole grid
- [ ] **TRAY-09**: Popover closes on focus loss

### Settings & Credentials

- [ ] **SET-01**: Settings window allows user to enter/edit per-provider API key
- [ ] **SET-02**: Settings window allows user to enable/disable individual providers
- [ ] **SET-03**: Settings window allows user to pick refresh cadence: manual / 1m / 2m / 5m / 15m
- [ ] **SET-04**: Settings window has a Diagnostics tab showing tray-host backend, secret-store backend, log path, app version
- [ ] **SET-05**: API keys persist via `oo7` Secret Service when available
- [ ] **SET-06**: API keys fall back to `oo7` encrypted-file backend when no Secret Service daemon is running
- [ ] **SET-07**: Settings window allows toggling XDG autostart (writes/removes `~/.config/autostart/codexbar.desktop`)
- [ ] **SET-08**: Settings UI follows system light/dark theme (CSS `prefers-color-scheme`)

### Lifecycle & Integration

- [ ] **LIFE-01**: Second launch of `codexbar` brings the running instance's popover to focus (single-instance via DBus name)
- [ ] **LIFE-02**: App degrades gracefully on Wayland: hotkey + always-on-top features mark themselves unavailable in UI rather than crashing
- [ ] **LIFE-03**: Quit from tray menu cleanly stops the poller and persists state before exit
- [ ] **LIFE-04**: Log rotation enforces 7-day retention (no unbounded log growth)

### CLI

- [ ] **CLI-01**: `codexbar usage -p <provider>` prints current usage snapshot to stdout (JSON format)
- [ ] **CLI-02**: `codexbar cost -p <provider>` prints rolling-30-day cost from local JSONL scan
- [ ] **CLI-03**: `codexbar --version` prints semver version
- [ ] **CLI-04**: `codexbar diagnose` prints tray backend / secret backend / XDG paths / log path

### Distribution

- [ ] **DIST-01**: CI builds `.deb` package on `ubuntu-22.04` GitHub Actions runner
- [ ] **DIST-02**: CI builds AppImage on the same runner
- [ ] **DIST-03**: `.deb` declares correct runtime deps: `libwebkit2gtk-4.1-0`, `libgtk-3-0`, `libayatana-appindicator3-1`
- [ ] **DIST-04**: `.deb` Recommends `gnome-shell-extension-appindicator` for GNOME compat
- [ ] **DIST-05**: AppImage runs on Ubuntu 22.04 + 24.04 without additional installs (other than webkitgtk if missing)
- [ ] **DIST-06**: Release artifacts are SHA-256-sidecar'd in the GitHub Releases attachments
- [ ] **DIST-07**: Release process is `gh release create` with semver tag, AppImage + `.deb` + checksums

### Testing & Quality (Linux-specific)

- [ ] **TEST-01**: Unit tests for every provider plugin's response parser run in CI on every PR
- [ ] **TEST-02**: Integration test exercises the full poller → UsageStore → IPC → frontend round-trip with a mocked HTTP server
- [ ] **TEST-03**: Smoke test launches the built binary headlessly on `ubuntu-22.04` CI under Xvfb and verifies the tray icon registers on a mocked StatusNotifierWatcher
- [ ] **TEST-04**: Manual UAT matrix executed on Ubuntu 22.04 GNOME Wayland, Ubuntu 24.04 GNOME Wayland, Kubuntu 24.04 Plasma Wayland, Fedora 40 GNOME Wayland — minimum
- [ ] **TEST-05**: AppImage and `.deb` install/uninstall cleanly without orphaning files

### Documentation

- [ ] **DOC-01**: README explains supported distros, install steps for `.deb` and AppImage, and the GNOME AppIndicator extension requirement
- [ ] **DOC-02**: README documents Wayland feature matrix (hotkey/floating-bar behavior)
- [ ] **DOC-03**: CONTRIBUTING explains the dev environment (rustup, `cargo tauri dev`, system deps)
- [ ] **DOC-04**: Release notes generated from conventional-commit log on every tagged release

## v2 Requirements (deferred)

| Category | Requirement |
|----------|-------------|
| Providers | Gemini, Copilot, Cursor (deferred from v1 — re-vendor from Win-CodexBar once trait surface is hardened) |
| Providers | 33 additional providers in upstream Win-CodexBar (abacus, alibaba, amp, antigravity, augment, bedrock, codebuff, commandcode, crof, deepseek, doubao, factory, jetbrains, kilo, kimi, kimik2, kiro, manus, mimo, minimax, mistral, nanogpt, ollama, opencode, opencodego, perplexity, stepfun, synthetic, venice, vertexai, warp, windsurf, zai, infini) |
| Distribution | Flatpak package using xdg-desktop-portals |
| Distribution | Custom signed apt repo at `apt.codexbar.dev` |
| Distribution | arm64 native builds for Ubuntu/Debian |
| Provider auth | OAuth device-flow path for Claude / Gemini / Copilot |
| Provider auth | Browser cookie auto-import (Firefox/Chrome profile read) |
| UI | Floating always-on-top usage bar with orientation/opacity/click-through (best-effort under Wayland) |
| UI | Global hotkey to toggle popover (best-effort under Wayland) |
| UI | Per-provider quota notifications (toast at 80% / 100%) |
| UI | Multi-provider "merge icons" composite glyph |
| Quality | Auto-update flow via Tauri updater + signed manifest |
| i18n | Locales beyond English |
| Telemetry | Opt-in crash reporting |

## Out of Scope

| Feature | Reason |
|---------|--------|
| Snap package | `system-tray` interface degraded under confinement; global-shortcut blocked; Wayland tray broken under strict mode |
| 32-bit i386 build | Ubuntu LTS dropped i386 desktop |
| Custom theming engine | System theme inheritance is correct Linux behavior |
| WidgetKit-style desktop widgets | No portable Linux mechanism |
| Sparkle delta auto-update | Tauri updater + AppImage update are idiomatic |
| Telemetry by default | Privacy-first stance; only opt-in later |
| Local model proxying / MITM | We report usage; we don't proxy traffic |
| Persistent telemetry-by-default | Privacy-first |
| WSL-specific build | WSL users can run the Windows port |
| Windows-style DPAPI cookie decrypt | No Linux equivalent; defer cookie auth path to v2 |

## Traceability

| Requirement | Phase | Status |
|-------------|-------|--------|
| CORE-01 | Phase 0 | Pending |
| CORE-02 | Phase 0 | Pending |
| CORE-03 | Phase 0 | Pending |
| CORE-04 | Phase 1 | Pending |
| CORE-05 | Phase 1 | Pending |
| CORE-06 | Phase 1 | Pending |
| PROV-01 | Phase 1 | Pending |
| PROV-02 | Phase 1 | Pending |
| PROV-03 | Phase 1 | Pending |
| PROV-04 | Phase 1 | Pending |
| PROV-05 | Phase 1 | Pending |
| PROV-06 | Phase 1 | Pending |
| PROV-07 | Phase 1 | Pending |
| PROV-08 | Phase 1 | Pending |
| PROV-09 | Phase 1 | Pending |
| CLI-01 | Phase 1 | Pending |
| CLI-02 | Phase 1 | Pending |
| CLI-03 | Phase 1 | Pending |
| CLI-04 | Phase 1 | Pending |
| TEST-01 | Phase 1 | Pending |
| TRAY-01 | Phase 2 | Pending |
| TRAY-02 | Phase 2 | Pending |
| TRAY-03 | Phase 2 | Pending |
| TRAY-04 | Phase 2 | Pending |
| TRAY-05 | Phase 2 | Pending |
| TRAY-06 | Phase 2 | Pending |
| TRAY-07 | Phase 2 | Pending |
| TRAY-08 | Phase 2 | Pending |
| TRAY-09 | Phase 2 | Pending |
| LIFE-01 | Phase 2 | Pending |
| LIFE-02 | Phase 2 | Pending |
| SET-01 | Phase 3 | Pending |
| SET-02 | Phase 3 | Pending |
| SET-03 | Phase 3 | Pending |
| SET-04 | Phase 3 | Pending |
| SET-05 | Phase 3 | Pending |
| SET-06 | Phase 3 | Pending |
| SET-07 | Phase 3 | Pending |
| SET-08 | Phase 3 | Pending |
| LIFE-03 | Phase 3 | Pending |
| LIFE-04 | Phase 3 | Pending |
| TEST-02 | Phase 3 | Pending |
| DIST-01 | Phase 4 | Pending |
| DIST-02 | Phase 4 | Pending |
| DIST-03 | Phase 4 | Pending |
| DIST-04 | Phase 4 | Pending |
| DIST-05 | Phase 4 | Pending |
| DIST-06 | Phase 4 | Pending |
| DIST-07 | Phase 4 | Pending |
| TEST-03 | Phase 4 | Pending |
| TEST-04 | Phase 4 | Pending |
| TEST-05 | Phase 4 | Pending |
| DOC-01 | Phase 4 | Pending |
| DOC-02 | Phase 4 | Pending |
| DOC-03 | Phase 4 | Pending |
| DOC-04 | Phase 4 | Pending |

**Coverage:**
- v1 requirements: 56 total (alpha scope tightened to 5 providers: PROV-* renumbered 01-09; 3 deferred to v2)
- Mapped to phases: 56/56
- Unmapped: 0

---
*Requirements defined: 2026-05-17*
*Last updated: 2026-05-17 — traceability table populated by gsd-roadmapper*
