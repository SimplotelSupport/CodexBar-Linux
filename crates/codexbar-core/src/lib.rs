//! Headless core of CodexBar Linux.
//!
//! Selectively vendored from `Finesssee/Win-CodexBar`'s `rust/` crate (MIT).
//! Zero GUI / zero Windows-specific dependencies — designed to compile clean on
//! `x86_64-unknown-linux-gnu` and to be reused by the Tauri shell, the CLI
//! binary, and tests.

pub mod cli;
pub mod core;
pub mod cost_scanner;
pub mod keyring;
pub mod logging;
pub mod providers;
pub mod secure_file;
pub mod settings;
pub mod status;
