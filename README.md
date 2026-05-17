# CodexBar Linux

[![release](https://img.shields.io/github/v/release/sidhartha1s/CodexBar-Linux?include_prereleases)](https://github.com/sidhartha1s/CodexBar-Linux/releases)
[![license](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)

A native Linux/Ubuntu port of [steipete/CodexBar](https://github.com/steipete/CodexBar) (macOS menubar) and [Finesssee/Win-CodexBar](https://github.com/Finesssee/Win-CodexBar) (Windows system tray).

CodexBar surfaces real-time AI coding-provider usage limits, quota reset countdowns, and cumulative cost — so you can see exactly how much of your AI budget remains without opening a browser.

## Status: v0.2.0-alpha.4 (Tauri tray + popover, credential-wired)

v0.2.0-alpha.4 ships **both**:

- **`CodexBar`** — Tauri tray app with a popover that shows per-provider usage bars, reset countdowns, and cumulative cost. StatusNotifierItem + libayatana-appindicator3. Background poller refreshes every 2 minutes.
- **`codexbar`** — headless CLI from v0.1.0-alpha, unchanged: `codexbar usage`, `codexbar cost`, `codexbar config`, etc.

**Providers:** Codex, Claude, GitHub Copilot, OpenAI API, OpenRouter.
**Deferred to v0.3.0:** Settings UI with credential editing, Secret Service / `oo7` integration, the remaining ~36 providers from upstream.

## Install

### Desktop app (.deb — recommended)

```bash
curl -LO https://github.com/sidhartha1s/CodexBar-Linux/releases/latest/download/CodexBar_0.2.0-alpha.4_amd64.deb
sudo dpkg -i CodexBar_0.2.0-alpha.4_amd64.deb
sudo apt -f install   # auto-installs missing runtime deps if any
codexbar-desktop &    # launches into tray
```

### Desktop app (AppImage)

```bash
curl -LO https://github.com/sidhartha1s/CodexBar-Linux/releases/latest/download/CodexBar_0.2.0-alpha.4_amd64.AppImage
chmod +x CodexBar_*.AppImage
./CodexBar_*.AppImage
```

### CLI only (.deb)

```bash
curl -LO https://github.com/sidhartha1s/CodexBar-Linux/releases/latest/download/codexbar_0.2.0.alpha.4-1_amd64.deb
sudo dpkg -i codexbar_0.2.0.alpha.4-1_amd64.deb
```

### CLI tarball (any glibc-2.35+ Linux on amd64)

```bash
curl -L https://github.com/sidhartha1s/CodexBar-Linux/releases/latest/download/codexbar-0.2.0-alpha.4-linux-amd64.tar.gz | tar xz
sudo install -m 0755 codexbar-0.2.0-alpha.4-linux-amd64/codexbar /usr/local/bin/
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

### Storing provider credentials (v0.2.0-alpha.4+)

The desktop popover and CLI both read tokens from `~/.config/codexbar/token-accounts.json` (managed via the `codexbar account` subcommands — no plaintext editing required):

```bash
# GitHub Personal Access Token (for Copilot usage)
codexbar account add copilot --label main --token <github-pat>

# OpenRouter API key
codexbar account add openrouter --label main --token sk-or-...

# OpenAI API key
codexbar account add openaiapi --label main --token sk-...

# Claude session cookie or OAuth token
codexbar account add claude --label main --token 'sessionKey=...'

# Inspect / switch / remove
codexbar account list openrouter
codexbar account switch openrouter <label-or-id>
codexbar account remove openrouter main
```

The desktop popover picks up new credentials on its next 2-minute poll (or hit ↻ in the tray). Codex needs no token — it probes the local `codex` CLI binary directly. v0.3.0 will migrate this store onto the Secret Service via `oo7`.

## Build from source

Requires Rust 1.85+ (edition 2024) and a Linux glibc 2.35+ system (Ubuntu 22.04+).

```bash
# CLI only — no system deps required
git clone https://github.com/sidhartha1s/CodexBar-Linux
cd CodexBar-Linux
cargo build --release -p codexbar-core
./target/release/codexbar --version
```

For the **Tauri desktop app**, install GTK/WebKit system deps first:

```bash
sudo apt update && sudo apt install -y \
  libwebkit2gtk-4.1-dev \
  libgtk-3-dev \
  libayatana-appindicator3-dev \
  librsvg2-dev \
  libsoup-3.0-dev \
  build-essential pkg-config libssl-dev

cargo install tauri-cli --locked --version '^2'
cd apps/desktop-tauri && cargo tauri build --bundles deb appimage
```

## License

MIT. Headless modules and the 5 alpha providers are selectively vendored from [Finesssee/Win-CodexBar](https://github.com/Finesssee/Win-CodexBar) with full attribution preserved in `crates/codexbar-core/LICENSE-WIN-CODEXBAR` and `NOTICE`.

## Roadmap

- **v0.1.0-alpha** — headless CLI, 5 providers, `.deb` + `.tar.gz`
- **v0.2.0-alpha.4** (this release) — Tauri tray + popover with live provider grid + background poller; `codexbar account add` works for all 5 alpha providers (Claude, Codex via local CLI, Copilot, OpenAIApi, OpenRouter); credentials loaded from `~/.config/codexbar/token-accounts.json`
- **v0.3.0** — Settings UI with credential editing, Secret Service (`oo7`) keyring, autostart polish
- **v0.4.0** — remaining ~36 providers from upstream Win-CodexBar
