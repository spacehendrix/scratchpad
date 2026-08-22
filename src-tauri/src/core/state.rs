use std::path::Path;

use crate::core::crypto::MasterKey;
use crate::core::error::{CoreError, CoreResult};
use crate::core::keychain::KeySource;
use crate::core::model::DocMeta;
use crate::core::store::Store;

/// Everything that exists only while unlocked. Dropping it zeroizes the key
/// and releases the store.
pub struct Session {
    pub key: MasterKey,
    pub store: Store,
    /// Decrypted metadata for every document — the source for listing and
    /// title/preview search. Kept in sync with the store on every mutation.
    pub catalog: Vec<DocMeta>,
}

/// Global application state, managed behind a `Mutex` by the shell layer.
pub enum AppState {
    Locked,
    Unlocked(Session),
}

impl AppState {
    pub fn is_unlocked(&self) -> bool {
        matches!(self, AppState::Unlocked(_))
    }

    /// Acquire the key (this is where the Touch ID sheet appears), open the
    /// store, and decrypt the catalog. No-op when already unlocked.
    pub fn unlock(&mut self, source: &dyn KeySource, data_dir: &Path) -> CoreResult<()> {
        if self.is_unlocked() {
            return Ok(());
        }
        let key = source.load_or_create()?;
        let store = Store::open(data_dir)?;
        let catalog = store.load_catalog(&key)?;
        *self = AppState::Unlocked(Session {
            key,
            store,
            catalog,
        });
        Ok(())
    }

    pub fn lock(&mut self) {
        *self = AppState::Locked;
    }

    pub fn session(&self) -> CoreResult<&Session> {
        match self {
            AppState::Unlocked(s) => Ok(s),
            AppState::Locked => Err(CoreError::Locked),
        }
    }

    pub fn session_mut(&mut self) -> CoreResult<&mut Session> {
        match self {
            AppState::Unlocked(s) => Ok(s),
            AppState::Locked => Err(CoreError::Locked),
        }
    }
}

impl Default for AppState {
    fn default() -> Self {
        AppState::Locked
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::keychain::InMemoryKeySource;
    use crate::core::model::{derive_checklist, derive_preview};

    fn meta(id: &str, body: &str) -> DocMeta {
        DocMeta {
            id: id.into(),
            title: None,
            preview: derive_preview(body),
            created_at: 0,
            updated_at: 0,
            pinned: false,
            archived_at: None,
            size_bytes: body.len() as u32,
            checklist: derive_checklist(body),
        }
    }

    #[test]
    fn unlock_lock_cycle() {
        let dir = tempfile::tempdir().unwrap();
        let source = InMemoryKeySource([1u8; 32]);
        let mut state = AppState::default();
        assert!(!state.is_unlocked());
        assert!(matches!(state.session(), Err(CoreError::Locked)));

        state.unlock(&source, dir.path()).unwrap();
        assert!(state.is_unlocked());
        assert!(state.session().unwrap().catalog.is_empty());

        // Unlock is idempotent.
        state.unlock(&source, dir.path()).unwrap();

        state.lock();
        assert!(!state.is_unlocked());
    }

    #[test]
    fn unlock_loads_existing_catalog() {
        let dir = tempfile::tempdir().unwrap();
        let source = InMemoryKeySource([1u8; 32]);
        {
            let mut state = AppState::default();
            state.unlock(&source, dir.path()).unwrap();
            let s = state.session_mut().unwrap();
            let m = meta("doc-1", "hello");
            s.store.upsert(&s.key, &m, "hello").unwrap();
        }
        let mut state = AppState::default();
        state.unlock(&source, dir.path()).unwrap();
        assert_eq!(state.session().unwrap().catalog.len(), 1);
    }

    #[test]
    fn unlock_with_wrong_key_reports_corrupt_and_stays_locked() {
        let dir = tempfile::tempdir().unwrap();
        {
            let mut state = AppState::default();
            state.unlock(&InMemoryKeySource([1u8; 32]), dir.path()).unwrap();
            let s = state.session_mut().unwrap();
            let m = meta("doc-1", "hello");
            s.store.upsert(&s.key, &m, "hello").unwrap();
        }
        let mut state = AppState::default();
        let err = state
            .unlock(&InMemoryKeySource([2u8; 32]), dir.path())
            .unwrap_err();
        assert!(matches!(err, CoreError::Corrupt { .. }));
        assert!(!state.is_unlocked());
    }
}
