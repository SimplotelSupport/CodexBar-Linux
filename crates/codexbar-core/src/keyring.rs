//! Minimal `keyring` API shim.
//!
//! Phase 0 of the Linux port vendors logic that originally relied on the
//! `keyring` crate (which doesn't fit our `oo7` Secret Service + encrypted-file
//! fallback plan). This module exposes the slice of the `keyring` crate's
//! public API the vendored code touches — `Entry::new`, `get_password`,
//! `set_password`, `delete_credential`, plus the `Error::NoEntry` /
//! `Error::Ambiguous` variants. Every operation returns
//! `Error::PlatformFailure`, so credential paths are wired but inert.
//!
//! Phase 3 replaces this shim with the real `oo7`-backed implementation.

#![allow(dead_code)]

use std::fmt;

/// Stand-in for `keyring::Entry`.
pub struct Entry {
    _service: String,
    _account: String,
}

impl Entry {
    pub fn new(service: &str, account: &str) -> Result<Self, Error> {
        Ok(Self {
            _service: service.to_owned(),
            _account: account.to_owned(),
        })
    }

    pub fn get_password(&self) -> Result<String, Error> {
        Err(Error::PlatformFailure(
            "credential storage not yet wired (Phase 3)".to_string(),
        ))
    }

    pub fn set_password(&self, _password: &str) -> Result<(), Error> {
        Err(Error::PlatformFailure(
            "credential storage not yet wired (Phase 3)".to_string(),
        ))
    }

    pub fn delete_credential(&self) -> Result<(), Error> {
        Err(Error::PlatformFailure(
            "credential storage not yet wired (Phase 3)".to_string(),
        ))
    }
}

/// Stand-in for `keyring::Error`. Only the variants the vendored code
/// pattern-matches against are exposed.
#[derive(Debug)]
pub enum Error {
    NoEntry,
    Ambiguous(Vec<()>),
    PlatformFailure(String),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::NoEntry => write!(f, "no credential entry found"),
            Error::Ambiguous(_) => write!(f, "ambiguous credential entry"),
            Error::PlatformFailure(msg) => write!(f, "credential platform failure: {msg}"),
        }
    }
}

impl std::error::Error for Error {}
