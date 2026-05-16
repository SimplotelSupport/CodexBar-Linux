# Roadmap: CodexBar Linux

## Overview

CodexBar Linux is a platform port of the Windows Tauri/Rust port to Linux. The journey: resolve the Cargo feature-gate blocker (Phase 0), build and validate the headless CLI core with all providers (Phase 1), add the visible Tauri tray+popover shell (Phase 2), wire live data and credentials end-to-end (Phase 3), then package and ship a production release (Phase 4). Each phase delivers one coherent, testable capability before the next begins.

## Phases

**Phase Numbering:**
- Integer phases (0, 1, 2, 3, 4): Planned milestone work
- Decimal phases (e.g., 2.1): Urgent insertions (marked with INSERTED)

Decimal phases appear between their surrounding integers in numeric order.

- [ ] **Phase 0: PRD & Foundation** - Resolve Cargo feature-gate blocker; establish headless-on-Linux workspace
- [ ] **Phase 1: Headless Core + CLI (Dev)** - All providers compile and test green on Linux; CLI binary works standalone
- [ ] **Phase 2: Tauri Shell + Tray (Dev)** - App launches; tray visible on GNOME + KDE; popover opens
- [ ] **Phase 3: Data Pipeline + Credentials (Dev to Testing)** - Live usage data flows to popover; settings and Secret Service wired; CI integration tests green
- [ ] **Phase 4: Distribution & Production Release** - `.deb` + AppImage built in CI; UAT complete; tagged GA release shipped

## Phase Details

### Phase 0: PRD & Foundation
**Goal**: Cargo workspace exists with `codexbar-core` selectively vendored from Win-CodexBar (headless modules + 5 alpha providers only); `cargo build` green on Linux with default features
**Depends on**: Nothing (first phase)
**Requirements**: CORE-01, CORE-02, CORE-03
**Success Criteria** (what must be TRUE):
  1. `cargo build -p codexbar-core` succeeds on `x86_64-unknown-linux-gnu` with default features (no feature-flag gymnastics)
  2. `cargo tree -p codexbar-core` contains zero matches for `winit`, `eframe`, `egui`, `tray-icon`, `muda`, `global-hotkey`, `keyring`, `aes-gcm`, `winreg`, or the `windows` crate
  3. `cargo tree -p codexbar-core` shows `rustls` and no `openssl-sys` or `native-tls`
  4. `crates/codexbar-core/LICENSE-WIN-CODEXBAR` exists with the upstream MIT license verbatim; top-level `NOTICE` credits Win-CodexBar
  5. `crates/codexbar-core/src/providers/` contains exactly: `codex`, `claude`, `openai`, `openaiapi`, `openrouter`, plus `mod.rs` — no other provider directories
**Plans**: TBD

### Phase 1: Headless Core + CLI (Dev)
**Goal**: All provider plugins compile and have passing fixture tests on Linux; the `codexbar` CLI binary works standalone on Linux
**Depends on**: Phase 0
**Requirements**: CORE-04, CORE-05, CORE-06, PROV-01, PROV-02, PROV-03, PROV-04, PROV-05, PROV-06, PROV-07, PROV-08, PROV-09, CLI-01, CLI-02, CLI-03, CLI-04, TEST-01
**Success Criteria** (what must be TRUE):
  1. `cargo test -p codexbar` passes on Linux CI with all provider fixture tests green
  2. `codexbar usage -p codex` prints a JSON snapshot to stdout on a machine with `~/.codex/sessions/` logs
  3. `codexbar cost -p claude` prints a 30-day rolling cost figure from local JSONL scan
  4. `codexbar --version` prints the semver version string
  5. JSON logs appear under `$XDG_STATE_HOME/codexbar/logs/` after running the binary, with files older than 7 days absent
**Plans**: TBD

### Phase 2: Tauri Shell + Tray (Dev)
**Goal**: The desktop application launches, registers a tray icon on GNOME and KDE, and opens the popover on click
**Depends on**: Phase 1
**Requirements**: TRAY-01, TRAY-02, TRAY-03, TRAY-04, TRAY-05, TRAY-06, TRAY-07, TRAY-08, TRAY-09, LIFE-01, LIFE-02
**Success Criteria** (what must be TRUE):
  1. Tray icon is visible in KDE Plasma 6 and GNOME 45+ (with AppIndicator extension) after `codexbar` launch
  2. Left-click on tray opens the provider grid popover; right-click shows Refresh / Settings / Quit menu
  3. Launching a second `codexbar` instance brings the running popover to focus instead of opening a duplicate
  4. On a Wayland session, hotkey and always-on-top features are marked unavailable in UI without crashing
  5. When no tray host is detected, app shows a non-blocking toast and remains keyboard-accessible
**Plans**: TBD
**UI hint**: yes

### Phase 3: Data Pipeline + Credentials (Dev to Testing)
**Goal**: Live provider usage data populates the popover and persists via Secret Service; settings window is fully functional; integration tests are green
**Depends on**: Phase 2
**Requirements**: SET-01, SET-02, SET-03, SET-04, SET-05, SET-06, SET-07, SET-08, LIFE-03, LIFE-04, TEST-02
**Success Criteria** (what must be TRUE):
  1. Entering an API key in Settings and clicking Save stores it in GNOME Keyring (or encrypted file fallback) — key survives app restart
  2. Popover usage bars update with real provider data within the configured polling interval (1m/2m/5m/15m or manual)
  3. Quit from tray menu cleanly stops the poller and writes state to disk (verified: relaunch shows last-known data within 100 ms)
  4. Integration test (`TEST-02`) runs green in CI: full poller → UsageStore → IPC → frontend round-trip with mocked HTTP server
**Plans**: TBD
**UI hint**: yes

### Phase 4: Distribution & Production Release
**Goal**: `.deb` and AppImage artifacts are built in CI, pass install/uninstall smoke tests, and a tagged GA release is published on GitHub Releases
**Depends on**: Phase 3
**Requirements**: DIST-01, DIST-02, DIST-03, DIST-04, DIST-05, DIST-06, DIST-07, TEST-03, TEST-04, TEST-05, DOC-01, DOC-02, DOC-03, DOC-04
**Success Criteria** (what must be TRUE):
  1. GitHub Actions produces `.deb` and AppImage artifacts on every push to `main` (no manual packaging step)
  2. `dpkg -I codexbar.deb` shows `libwebkit2gtk-4.1-0`, `libgtk-3-0`, `libayatana-appindicator3-1` as runtime deps
  3. Manual UAT passes on Ubuntu 22.04 GNOME Wayland, Ubuntu 24.04 GNOME Wayland, Kubuntu 24.04 Plasma Wayland, and Fedora 40 GNOME Wayland
  4. `gh release create` publishes `.deb`, AppImage, and SHA-256 sidecars under a semver tag; release notes generated from conventional-commit log
**Plans**: TBD

## Progress

**Execution Order:**
Phases execute in numeric order: 0 → 1 → 2 → 3 → 4

| Phase | Plans Complete | Status | Completed |
|-------|----------------|--------|-----------|
| 0. PRD & Foundation | 0/TBD | Not started | - |
| 1. Headless Core + CLI (Dev) | 0/TBD | Not started | - |
| 2. Tauri Shell + Tray (Dev) | 0/TBD | Not started | - |
| 3. Data Pipeline + Credentials (Dev to Testing) | 0/TBD | Not started | - |
| 4. Distribution & Production Release | 0/TBD | Not started | - |
