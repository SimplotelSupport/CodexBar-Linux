//! Provider implementations.
//!
//! Alpha (v0.1.0) ships with 5 providers (Codex, Claude, Copilot, OpenAI API,
//! OpenRouter). The 36 additional providers in the upstream Win-CodexBar
//! crate are tracked as v2 work — see `.planning/REQUIREMENTS.md`.

#![allow(dead_code)]

pub mod claude;
pub mod codex;
pub mod copilot;
pub mod openaiapi;
pub mod openrouter;

pub use claude::ClaudeProvider;
pub use codex::CodexProvider;
pub use copilot::CopilotProvider;
pub use openaiapi::OpenAIApiProvider;
pub use openrouter::OpenRouterProvider;
