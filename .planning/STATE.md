# Project State

## Project Reference

See: .planning/PROJECT.md (updated 2026-05-17)

**Core value:** A glanceable Linux tray indicator that tells a developer right now how much of their AI provider quota is left and when it resets — without leaving their editor.
**Current focus:** Phase 0 — PRD & Foundation

## Current Position

Phase: 0 of 4 (PRD & Foundation)
Plan: 0 of TBD in current phase
Status: Ready to plan
Last activity: 2026-05-17 — ROADMAP.md and STATE.md created; requirements traceability populated

Progress: [░░░░░░░░░░] 0%

## Performance Metrics

**Velocity:**
- Total plans completed: 0
- Average duration: -
- Total execution time: 0 hours

**By Phase:**

| Phase | Plans | Total | Avg/Plan |
|-------|-------|-------|----------|
| - | - | - | - |

**Recent Trend:**
- Last 5 plans: -
- Trend: -

*Updated after each plan completion*

## Accumulated Context

### Decisions

Decisions are logged in PROJECT.md Key Decisions table.
Recent decisions affecting current work:

- [Init]: Tauri v2 + Rust chosen; reuse Win-CodexBar `codexbar` crate feature-gated for Linux headless
- [Init]: `.deb` + AppImage for v1; Flatpak deferred
- [Init]: Secret Service API (`oo7`) primary; encrypted-file fallback for headless sessions
- [Init]: PRD → dev → testing → prod shape; coarse granularity (5 phases: 0–4)

### Pending Todos

None yet.

### Blockers/Concerns

- [Phase 0 BLOCKER] `codexbar` crate currently compiles GUI deps unconditionally — conflicts with Tauri's copies of `winit`/`tray-icon`. Must resolve before any Tauri code is written (P-01 from SUMMARY.md).
- [Phase 2 Watch] GNOME 45+ AppIndicator behavior varies by distro patch — needs VM validation during Phase 2.
- [Phase 4 Watch] webkitgtk 4.1 (Ubuntu 22.04) vs 6.0 (Ubuntu 24.04) packaging strategy — decision needed before Phase 4 CI.

## Deferred Items

| Category | Item | Status | Deferred At |
|----------|------|--------|-------------|
| Distribution | Flatpak package | v2 | Init |
| Distribution | Custom signed apt repo | v2 | Init |
| Distribution | arm64 native builds | v2 | Init |
| Provider auth | OAuth device-flow | v2 | Init |
| Provider auth | Browser cookie auto-import | v2 | Init |
| UI | Floating always-on-top bar | v2 | Init |
| UI | Global hotkey (Wayland) | v2 | Init |

## Session Continuity

Last session: 2026-05-17
Stopped at: Roadmap created; traceability table populated in REQUIREMENTS.md
Resume file: None
