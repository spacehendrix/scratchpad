//! Authenticated encryption for everything that touches disk.
//!
//! XChaCha20-Poly1305 with a random 24-byte nonce per blob (large enough that
//! random nonces can never realistically collide). Blob layout:
//! `nonce (24 bytes) || ciphertext || tag (16 bytes)`.
//!
//! Every blob is bound to its location via AAD (`"{doc_id}:{field}"`), so
//! ciphertexts cannot be swapped between rows or between the meta/body
//! columns without failing authentication.

use chacha20poly1305::{
    aead::{Aead, AeadCore, KeyInit, OsRng, Payload},
    XChaCha20Poly1305, XNonce,
};
use zeroize::Zeroizing;

pub const KEY_LEN: usize = 32;
pub const NONCE_LEN: usize = 24;

/// The 256-bit master key. Zeroized on drop; never leaves the core layer.
pub struct MasterKey(Zeroizing<[u8; KEY_LEN]>);

impl MasterKey {
    pub fn from_bytes(bytes: [u8; KEY_LEN]) -> Self {
        MasterKey(Zeroizing::new(bytes))
    }

    /// Generate a fresh random key (first-run only; the keychain owns it
    /// afterwards).
    pub fn generate() -> Self {
        let key = XChaCha20Poly1305::generate_key(&mut OsRng);
        MasterKey(Zeroizing::new(key.into()))
    }

    pub fn as_bytes(&self) -> &[u8; KEY_LEN] {
        &self.0
    }

    fn cipher(&self) -> XChaCha20Poly1305 {
        XChaCha20Poly1305::new(self.0.as_ref().into())
    }
}

/// Decryption/authentication failure. Deliberately carries no detail — the
/// caller knows which document it was working on.
#[derive(Debug, PartialEq, Eq)]
pub struct CryptoError;

/// AAD for a document field, e.g. `aad("018f…", "body")`.
pub fn aad(doc_id: &str, field: &str) -> Vec<u8> {
    format!("{doc_id}:{field}").into_bytes()
}

pub fn seal(key: &MasterKey, aad: &[u8], plaintext: &[u8]) -> Vec<u8> {
    let nonce = XChaCha20Poly1305::generate_nonce(&mut OsRng);
    let ciphertext = key
        .cipher()
        .encrypt(
            &nonce,
            Payload {
                msg: plaintext,
                aad,
            },
        )
        .expect("XChaCha20-Poly1305 encryption is infallible for in-memory buffers");
    let mut blob = Vec::with_capacity(NONCE_LEN + ciphertext.len());
    blob.extend_from_slice(&nonce);
    blob.extend_from_slice(&ciphertext);
    blob
}

pub fn open(key: &MasterKey, aad: &[u8], blob: &[u8]) -> Result<Vec<u8>, CryptoError> {
    if blob.len() < NONCE_LEN {
        return Err(CryptoError);
    }
    let (nonce, ciphertext) = blob.split_at(NONCE_LEN);
    key.cipher()
        .decrypt(
            XNonce::from_slice(nonce),
            Payload {
                msg: ciphertext,
                aad,
            },
        )
        .map_err(|_| CryptoError)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrip() {
        let key = MasterKey::generate();
        let aad = aad("doc-1", "body");
        let blob = seal(&key, &aad, b"hello scratchpad");
        assert_eq!(open(&key, &aad, &blob).unwrap(), b"hello scratchpad");
    }

    #[test]
    fn nonces_are_unique_per_seal() {
        let key = MasterKey::generate();
        let a = seal(&key, b"x", b"same");
        let b = seal(&key, b"x", b"same");
        assert_ne!(a, b);
        assert_ne!(&a[..NONCE_LEN], &b[..NONCE_LEN]);
    }

    #[test]
    fn tampered_blob_is_rejected() {
        let key = MasterKey::generate();
        let mut blob = seal(&key, b"x", b"payload");
        let last = blob.len() - 1;
        blob[last] ^= 0x01;
        assert_eq!(open(&key, b"x", &blob), Err(CryptoError));
    }

    #[test]
    fn wrong_aad_is_rejected() {
        let key = MasterKey::generate();
        let blob = seal(&key, &aad("doc-1", "body"), b"payload");
        assert_eq!(open(&key, &aad("doc-1", "meta"), &blob), Err(CryptoError));
        assert_eq!(open(&key, &aad("doc-2", "body"), &blob), Err(CryptoError));
    }

    #[test]
    fn wrong_key_is_rejected() {
        let blob = seal(&MasterKey::generate(), b"x", b"payload");
        assert_eq!(open(&MasterKey::generate(), b"x", &blob), Err(CryptoError));
    }

    #[test]
    fn truncated_blob_is_rejected() {
        let key = MasterKey::generate();
        let blob = seal(&key, b"x", b"payload");
        assert_eq!(open(&key, b"x", &blob[..10]), Err(CryptoError));
        assert_eq!(open(&key, b"x", &[]), Err(CryptoError));
    }
}
