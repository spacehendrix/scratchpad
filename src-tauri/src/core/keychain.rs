//! Where the master key comes from. The real macOS keychain source (Touch ID
//! gated) arrives in the unlock milestone; tests and the store layer only
//! depend on the trait.

use crate::core::crypto::{MasterKey, KEY_LEN};
use crate::core::error::CoreResult;

pub trait KeySource {
    /// Return the master key, creating and persisting a fresh one on first
    /// run. May block on user interaction (Touch ID / password sheet).
    fn load_or_create(&self) -> CoreResult<MasterKey>;
}

/// Deterministic key source for tests — no keychain, no prompts.
pub struct InMemoryKeySource(pub [u8; KEY_LEN]);

impl KeySource for InMemoryKeySource {
    fn load_or_create(&self) -> CoreResult<MasterKey> {
        Ok(MasterKey::from_bytes(self.0))
    }
}
