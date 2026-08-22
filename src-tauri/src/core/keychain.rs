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

/// macOS keychain source. The key lives as a generic-password item in the
/// Data Protection keychain with `kSecAccessControlUserPresence`: reading it
/// makes the OS show the Touch ID sheet (with the account-password fallback),
/// and because the item is a random key — not derived from the login
/// password — it survives macOS password changes.
#[cfg(target_os = "macos")]
pub struct KeychainKeySource {
    service: String,
    account: String,
}

#[cfg(target_os = "macos")]
impl KeychainKeySource {
    pub fn new() -> Self {
        KeychainKeySource {
            service: "com.spacehendrix.scratchpad".into(),
            account: "master-key".into(),
        }
    }
}

#[cfg(target_os = "macos")]
impl KeySource for KeychainKeySource {
    fn load_or_create(&self) -> CoreResult<MasterKey> {
        use crate::core::error::CoreError;
        use security_framework::base::Error as SecError;
        use security_framework::passwords::{get_generic_password, set_generic_password_options};
        use security_framework::passwords_options::{AccessControlOptions, PasswordOptions};

        // OSStatus codes (Security/SecBase.h).
        const ERR_SEC_ITEM_NOT_FOUND: i32 = -25300;
        const ERR_SEC_USER_CANCELED: i32 = -128;
        const ERR_SEC_AUTH_FAILED: i32 = -25293;
        const ERR_SEC_INTERACTION_NOT_ALLOWED: i32 = -25308;

        let map_err = |e: SecError| match e.code() {
            ERR_SEC_USER_CANCELED | ERR_SEC_AUTH_FAILED | ERR_SEC_INTERACTION_NOT_ALLOWED => {
                CoreError::KeychainDenied
            }
            _ => CoreError::Io(format!("keychain error {}", e.code())),
        };

        match get_generic_password(&self.service, &self.account) {
            Ok(bytes) => {
                let bytes: [u8; KEY_LEN] = bytes
                    .as_slice()
                    .try_into()
                    .map_err(|_| CoreError::KeychainItemMissing)?;
                Ok(MasterKey::from_bytes(bytes))
            }
            Err(e) if e.code() == ERR_SEC_ITEM_NOT_FOUND => {
                let key = MasterKey::generate();
                let mut options =
                    PasswordOptions::new_generic_password(&self.service, &self.account);
                options.set_access_control_options(AccessControlOptions::USER_PRESENCE);
                set_generic_password_options(key.as_bytes(), options).map_err(map_err)?;
                Ok(key)
            }
            Err(e) => Err(map_err(e)),
        }
    }
}
