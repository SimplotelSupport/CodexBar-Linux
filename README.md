# CodexBar Linux

[![release](https://img.shields.io/github/v/release/sidhartha1s/CodexBar-Linux?include_prereleases)](https://github.com/sidhartha1s/CodexBar-Linux/releases)
[![license](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)

A native Linux/Ubuntu port of [steipete/CodexBar](https://github.com/steipete/CodexBar) (macOS menubar) and [Finesssee/Win-CodexBar](https://github.com/Finesssee/Win-CodexBar) (Windows system tray).

CodexBar surfaces real-time AI coding-provider usage limits, quota reset countdowns, and cumulative cost — so you can see exactly how much of your AI budget remains without opening a browser.

## Status: v0.2.0-alpha (Tauri tray + popover)

v0.2.0-alpha ships **both**:

- **`CodexBar`** — Tauri tray app with a popover that shows per-provider usage bars, reset countdowns, and cumulative cost. StatusNotifierItem + libayatana-appindicator3. Background poller refreshes every 2 minutes.
- **`codexbar`** — headless CLI from v0.1.0-alpha, unchanged: `codexbar usage`, `codexbar cost`, `codexbar config`, etc.

**Providers:** Codex, Claude, GitHub Copilot, OpenAI API, OpenRouter.
**Deferred to v0.3.0:** Settings UI with credential editing, Secret Service / `oo7` integration, the remaining ~36 providers from upstream.

## Install

### Desktop app (.deb — recommended)

```bash
# Pick whichever .deb appears under Releases (filename varies with version)
curl -LO https://github.com/sidhartha1s/CodexBar-Linux/releases/latest/download/CodexBar_0.2.0-alpha_amd64.deb
sudo dpkg -i CodexBar_0.2.0-alpha_amd64.deb
sudo apt -f install   # auto-installs missing runtime deps if any
codexbar-desktop &    # launches into tray
```

### Desktop app (AppImage)

```bash
curl -LO https://github.com/sidhartha1s/CodexBar-Linux/releases/latest/download/CodexBar_0.2.0-alpha_amd64.AppImage
chmod +x CodexBar_*.AppImage
./CodexBar_*.AppImage
```

### CLI only (.deb)

```bash
curl -LO https://github.com/sidhartha1s/CodexBar-Linux/releases/latest/download/codexbar_0.2.0-alpha-1_amd64.deb
sudo dpkg -i codexbar_0.2.0-alpha-1_amd64.deb
```

### CLI tarball (any glibc-2.35+ Linux on amd64)

```bash
curl -L https://github.com/sidhartha1s/CodexBar-Linux/releases/latest/download/codexbar-0.2.0-alpha-linux-amd64.tar.gz | tar xz
sudo install -m 0755 codexbar-0.2.0-alpha-linux-amd64/codexbar /usr/local/bin/
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
- **v0.2.0-alpha** (this release) — Tauri tray + popover with live provider grid + background poller
- **v0.3.0** — Settings UI with credential editing, Secret Service (`oo7`) keyring, autostart polish
- **v0.4.0** — remaining ~36 providers from upstream Win-CodexBar
