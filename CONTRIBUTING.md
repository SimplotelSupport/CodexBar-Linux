# Contributing to CodexBar-Linux

Thanks for your interest in contributing!

## Quick start

1. Fork the repo and create a branch from `main`.
2. Build with `cargo build` (CLI) — see `apps/` for the Tauri tray app.
3. Run `cargo fmt` and `cargo clippy` before committing.
4. Open a pull request with a clear description of WHY the change is needed.

## Guidelines

- Keep PRs small and focused — one logical change per PR.
- Provider additions should follow the existing provider structure in `crates/`.
- Bug reports: please include your distro, desktop environment, and `codexbar --version` output.

## License

By contributing, you agree that your contributions will be licensed under the same license as the project (see `LICENSE`).
