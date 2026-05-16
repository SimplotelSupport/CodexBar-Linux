//! Canonical provider factory.
//!
//! Alpha (v0.1.0) registers 5 providers. Other `ProviderId` variants remain in
//! the enum for forward-compat but are not yet instantiable — adding them in
//! v2 means re-vendoring the corresponding `providers/<name>/` directory and
//! adding a match arm here.

use super::{Provider, ProviderId};
use crate::providers::{
    ClaudeProvider, CodexProvider, CopilotProvider, OpenAIApiProvider, OpenRouterProvider,
};

/// Instantiate the concrete [`Provider`] implementation for a given [`ProviderId`].
///
/// Returns `None` for providers not yet implemented in this alpha. Callers
/// should treat `None` as "provider deferred to v2".
pub fn try_instantiate(id: ProviderId) -> Option<Box<dyn Provider>> {
    match id {
        ProviderId::Codex => Some(Box::new(CodexProvider::new())),
        ProviderId::Claude => Some(Box::new(ClaudeProvider::new())),
        ProviderId::Copilot => Some(Box::new(CopilotProvider::new())),
        ProviderId::OpenAIApi => Some(Box::new(OpenAIApiProvider::new())),
        ProviderId::OpenRouter => Some(Box::new(OpenRouterProvider::new())),
        _ => None,
    }
}

/// Instantiate the concrete [`Provider`] implementation, panicking for IDs
/// not yet wired in. Use [`try_instantiate`] when callers can degrade
/// gracefully.
pub fn instantiate(id: ProviderId) -> Box<dyn Provider> {
    try_instantiate(id)
        .unwrap_or_else(|| panic!("provider {id:?} is not implemented in this alpha"))
}

/// All provider IDs that are wired up in this build.
pub fn alpha_provider_ids() -> &'static [ProviderId] {
    &[
        ProviderId::Codex,
        ProviderId::Claude,
        ProviderId::Copilot,
        ProviderId::OpenAIApi,
        ProviderId::OpenRouter,
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_alpha_provider_id_is_instantiable() {
        for &id in alpha_provider_ids() {
            let provider = instantiate(id);
            assert_eq!(
                provider.id(),
                id,
                "factory returned wrong provider for {id:?}"
            );
        }
    }
}
