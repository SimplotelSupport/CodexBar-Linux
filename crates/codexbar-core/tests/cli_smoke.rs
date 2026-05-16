//! Smoke tests for the `codexbar` CLI binary.

use std::process::Command;

fn bin() -> std::path::PathBuf {
    let mut p = std::path::PathBuf::from(env!("CARGO_BIN_EXE_codexbar"));
    if !p.exists() {
        p = std::path::PathBuf::from(env!("CARGO_TARGET_TMPDIR"))
            .parent()
            .unwrap()
            .join("debug/codexbar");
    }
    p
}

#[test]
fn version_flag_prints_semver() {
    let out = Command::new(bin())
        .arg("--version")
        .output()
        .expect("spawn codexbar");
    assert!(
        out.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    let s = String::from_utf8_lossy(&out.stdout);
    let expected = env!("CARGO_PKG_VERSION");
    assert!(s.contains(expected), "got: {s}, expected version {expected}");
}

#[test]
fn help_lists_alpha_subcommands() {
    let out = Command::new(bin())
        .arg("--help")
        .output()
        .expect("spawn codexbar");
    assert!(out.status.success());
    let s = String::from_utf8_lossy(&out.stdout);
    for cmd in ["usage", "cost", "autostart", "account", "config"] {
        assert!(s.contains(cmd), "help missing subcommand {cmd}: {s}");
    }
}

#[test]
fn provider_help_lists_alpha_providers() {
    let out = Command::new(bin())
        .arg("--help")
        .output()
        .expect("spawn codexbar");
    let s = String::from_utf8_lossy(&out.stdout);
    for p in ["codex", "claude", "copilot", "openaiapi", "openrouter"] {
        assert!(s.contains(p), "provider {p} missing from help: {s}");
    }
}
