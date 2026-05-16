# CodexBar Linux

[![release](https://img.shields.io/github/v/release/sidhartha1s/CodexBar-Linux?include_prereleases)](https://github.com/sidhartha1s/CodexBar-Linux/releases)
[![license](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)

A native Linux/Ubuntu port of [steipete/CodexBar](https://github.com/steipete/CodexBar) (macOS menubar) and [Finesssee/Win-CodexBar](https://github.com/Finesssee/Win-CodexBar) (Windows system tray).

CodexBar surfaces real-time AI coding-provider usage limits, quota reset countdowns, and cumulative cost — so you can see exactly how much of your AI budget remains without opening a browser.

## Status: v0.1.0-alpha (CLI-only)

This first alpha ships the **headless `codexbar` CLI binary**. It validates the cross-cutting headless core (HTTP, JSONL cost scan, provider abstraction, settings, logging) ahead of the v0.2.0 Tauri tray + popover.

**Providers in this alpha:** Codex, Claude, GitHub Copilot, OpenAI API, OpenRouter.
**Deferred to v0.2.0:** tray icon, popover, Settings UI, Secret Service credentials, autostart.
**Deferred to v0.3.0:** the remaining ~36 providers from upstream.

## Install

### Debian/Ubuntu (.deb)

```bash
curl -L https://github.com/sidhartha1s/CodexBar-Linux/releases/download/v0.1.0-alpha/codexbar_0.1.0-alpha_amd64.deb -o codexbar.deb
sudo dpkg -i codexbar.deb
```

### Tarball (any glibc-2.35+ Linux on amd64)

```bash
curl -L https://github.com/sidhartha1s/CodexBar-Linux/releases/download/v0.1.0-alpha/codexbar-0.1.0-alpha-linux-amd64.tar.gz | tar xz
sudo install -m 0755 codexbar /usr/local/bin/
```

Verify integrity with the published `.sha256` sidecars.

## Usage

```bash
codexbar --version
codexbar --help

# Print usage for a single provider as text
codexbar usage -p claude

# Print usage as JSON for scripting
codexbar usage -p codex --json

# Local 30-day cost scan from Codex/Claude JSONL logs
codexbar cost -p claude
codexbar cost -p codex
```

## Build from source

Requires Rust 1.85+ (edition 2024) and a Linux glibc 2.35+ system (Ubuntu 22.04+).

```bash
git clone https://github.com/sidhartha1s/CodexBar-Linux
cd CodexBar-Linux
cargo build --release -p codexbar-core
./target/release/codexbar --version
```

## License

MIT. Headless modules and the 5 alpha providers are selectively vendored from [Finesssee/Win-CodexBar](https://github.com/Finesssee/Win-CodexBar) with full attribution preserved in `crates/codexbar-core/LICENSE-WIN-CODEXBAR` and `NOTICE`.

## Roadmap

- **v0.1.0-alpha** (this release) — headless CLI, 5 providers, `.deb` + `.tar.gz`
- **v0.2.0** — Tauri shell + tray icon + popover + Settings UI + Secret Service credentials
- **v0.3.0** — remaining ~36 providers from upstream Win-CodexBar
