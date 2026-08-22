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

/// macOS keychain source.
///
/// The master key lives as a generic-password item in the user's login
/// keychain, and every read is gated behind an explicit LocalAuthentication
/// user-presence check (`LAPolicy::DeviceOwnerAuthentication` — Touch ID
/// with the account-password fallback, the same system sheet the Data
/// Protection keychain would show). Because the key is random — not derived
/// from the login password — it survives macOS password changes.
///
/// Why not the Data Protection keychain with `kSecAccessControlUserPresence`?
/// It refuses items from binaries without a proper signing entitlement
/// (OSStatus -34018), and this app is local-build only (ad-hoc signed). A
/// try-DP-first strategy was rejected deliberately: two possible key
/// locations would let a differently-signed future build silently mint a
/// second key and orphan the database. One location, one key.
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

/// Block until the user passes the system Touch ID / password sheet.
#[cfg(target_os = "macos")]
fn authenticate_user_presence() -> CoreResult<()> {
    use crate::core::error::CoreError;
    use block2::RcBlock;
    use objc2::runtime::Bool;
    use objc2_foundation::{NSError, NSString};
    use objc2_local_authentication::{LAContext, LAPolicy};
    use std::sync::mpsc;

    let (tx, rx) = mpsc::channel::<bool>();
    let reply = RcBlock::new(move |success: Bool, _error: *mut NSError| {
        let _ = tx.send(success.as_bool());
    });
    unsafe {
        let ctx = LAContext::new();
        ctx.evaluatePolicy_localizedReason_reply(
            LAPolicy::DeviceOwnerAuthentication,
            &NSString::from_str("unlock your scratchpad"),
            &reply,
        );
    }
    match rx.recv() {
        Ok(true) => Ok(()),
        _ => Err(CoreError::KeychainDenied),
    }
}

#[cfg(target_os = "macos")]
impl KeySource for KeychainKeySource {
    fn load_or_create(&self) -> CoreResult<MasterKey> {
        use crate::core::error::CoreError;
        use security_framework::passwords::{get_generic_password, set_generic_password};

        // OSStatus codes (Security/SecBase.h).
        const ERR_SEC_ITEM_NOT_FOUND: i32 = -25300;
        const ERR_SEC_USER_CANCELED: i32 = -128;
        const ERR_SEC_AUTH_FAILED: i32 = -25293;
        const ERR_SEC_INTERACTION_NOT_ALLOWED: i32 = -25308;

        match get_generic_password(&self.service, &self.account) {
            Ok(bytes) => {
                let bytes: [u8; KEY_LEN] = bytes
                    .as_slice()
                    .try_into()
                    .map_err(|_| CoreError::KeychainItemMissing)?;
                // The item exists: require user presence before handing the
                // key to the session.
                authenticate_user_presence()?;
                Ok(MasterKey::from_bytes(bytes))
            }
            // First run: create the key silently (there is nothing to
            // protect yet); every later unlock prompts.
            Err(e) if e.code() == ERR_SEC_ITEM_NOT_FOUND => {
                let key = MasterKey::generate();
                set_generic_password(&self.service, &self.account, key.as_bytes())
                    .map_err(|e| CoreError::Io(format!("keychain error {}", e.code())))?;
                Ok(key)
            }
            Err(e)
                if matches!(
                    e.code(),
                    ERR_SEC_USER_CANCELED | ERR_SEC_AUTH_FAILED | ERR_SEC_INTERACTION_NOT_ALLOWED
                ) =>
            {
                Err(CoreError::KeychainDenied)
            }
            Err(e) => Err(CoreError::Io(format!("keychain error {}", e.code()))),
        }
    }
}
