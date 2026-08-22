use std::path::Path;

use crate::core::crypto::MasterKey;
use crate::core::error::{CoreError, CoreResult};
use crate::core::keychain::KeySource;
use crate::core::model::{DocMeta, Document, UnixMs};
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

impl Session {
    /// Active documents for the browse list: pinned first, then most
    /// recently edited. Archived documents are only reachable via search.
    pub fn list(&self) -> Vec<DocMeta> {
        let mut docs: Vec<DocMeta> = self
            .catalog
            .iter()
            .filter(|m| m.archived_at.is_none())
            .cloned()
            .collect();
        docs.sort_by(|a, b| {
            b.pinned
                .cmp(&a.pinned)
                .then(b.updated_at.cmp(&a.updated_at))
        });
        docs
    }

    /// Load a full document. Deliberately does NOT bump any timestamp:
    /// reading is not "meaningful activity" for retention.
    pub fn get(&self, id: &str) -> CoreResult<Document> {
        let meta = self
            .catalog
            .iter()
            .find(|m| m.id == id)
            .cloned()
            .ok_or(CoreError::NotFound)?;
        let body = self.store.load_body(&self.key, id)?;
        Ok(Document {
            id: meta.id.clone(),
            title: meta.title.clone(),
            body,
            meta,
        })
    }

    /// Create (`id: None`) or update a document. Editing counts as activity:
    /// bumps `updated_at` and un-archives.
    pub fn save(
        &mut self,
        id: Option<String>,
        title: Option<String>,
        body: String,
        now: UnixMs,
    ) -> CoreResult<DocMeta> {
        use crate::core::model::{derive_checklist, derive_preview};

        let title = title.filter(|t| !t.trim().is_empty());
        let meta = match id {
            None => DocMeta {
                id: uuid::Uuid::now_v7().to_string(),
                title,
                preview: derive_preview(&body),
                created_at: now,
                updated_at: now,
                pinned: false,
                archived_at: None,
                size_bytes: body.len() as u32,
                checklist: derive_checklist(&body),
            },
            Some(id) => {
                let existing = self
                    .catalog
                    .iter()
                    .find(|m| m.id == id)
                    .ok_or(CoreError::NotFound)?;
                DocMeta {
                    id: existing.id.clone(),
                    title,
                    preview: derive_preview(&body),
                    created_at: existing.created_at,
                    updated_at: now,
                    pinned: existing.pinned,
                    archived_at: None, // an edit un-archives
                    size_bytes: body.len() as u32,
                    checklist: derive_checklist(&body),
                }
            }
        };

        self.store.upsert(&self.key, &meta, &body)?;
        self.put_catalog(meta.clone());
        Ok(meta)
    }

    /// Pinning is not a content edit — `updated_at` is untouched.
    pub fn toggle_pin(&mut self, id: &str) -> CoreResult<DocMeta> {
        let mut meta = self
            .catalog
            .iter()
            .find(|m| m.id == id)
            .cloned()
            .ok_or(CoreError::NotFound)?;
        meta.pinned = !meta.pinned;
        self.store.update_meta(&self.key, &meta)?;
        self.put_catalog(meta.clone());
        Ok(meta)
    }

    pub fn delete(&mut self, id: &str) -> CoreResult<()> {
        self.store.delete(id)?;
        self.catalog.retain(|m| m.id != id);
        Ok(())
    }

    fn put_catalog(&mut self, meta: DocMeta) {
        match self.catalog.iter_mut().find(|m| m.id == meta.id) {
            Some(slot) => *slot = meta,
            None => self.catalog.push(meta),
        }
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

    fn unlocked(dir: &std::path::Path) -> AppState {
        let mut state = AppState::default();
        state.unlock(&InMemoryKeySource([1u8; 32]), dir).unwrap();
        state
    }

    #[test]
    fn save_create_update_and_list_ordering() {
        let dir = tempfile::tempdir().unwrap();
        let mut state = unlocked(dir.path());
        let s = state.session_mut().unwrap();

        let a = s.save(None, None, "note a".into(), 100).unwrap();
        let b = s.save(None, None, "[ ] task b".into(), 200).unwrap();
        let c = s.save(None, Some("titled".into()), "note c".into(), 300).unwrap();

        // Most recently edited first.
        let list = s.list();
        assert_eq!(
            list.iter().map(|m| m.id.as_str()).collect::<Vec<_>>(),
            vec![c.id.as_str(), b.id.as_str(), a.id.as_str()]
        );
        assert_eq!(list[0].title.as_deref(), Some("titled"));
        assert_eq!(list[1].checklist.unwrap().total, 1);

        // Pinned floats above newer docs.
        s.toggle_pin(&a.id).unwrap();
        let list = s.list();
        assert_eq!(list[0].id, a.id);
        assert!(list[0].pinned);
        // Pinning did not count as an edit.
        assert_eq!(list[0].updated_at, 100);

        // Updating an existing doc keeps id/created_at, bumps updated_at.
        let a2 = s.save(Some(a.id.clone()), None, "note a v2".into(), 400).unwrap();
        assert_eq!(a2.id, a.id);
        assert_eq!(a2.created_at, 100);
        assert_eq!(a2.updated_at, 400);
        assert_eq!(s.get(&a.id).unwrap().body, "note a v2");
    }

    #[test]
    fn edit_unarchives_but_get_does_not() {
        let dir = tempfile::tempdir().unwrap();
        let mut state = unlocked(dir.path());
        let s = state.session_mut().unwrap();
        let m = s.save(None, None, "old note".into(), 100).unwrap();

        // Simulate the retention pass having archived it.
        let mut archived = m.clone();
        archived.archived_at = Some(500);
        s.store.update_meta(&s.key, &archived).unwrap();
        s.catalog[0] = archived;
        assert!(s.list().is_empty());

        // Reading does not resurface it.
        s.get(&m.id).unwrap();
        assert!(s.list().is_empty());

        // Editing does.
        s.save(Some(m.id.clone()), None, "revived".into(), 600).unwrap();
        assert_eq!(s.list().len(), 1);
        assert!(s.list()[0].archived_at.is_none());
    }

    #[test]
    fn delete_removes_from_catalog_and_store() {
        let dir = tempfile::tempdir().unwrap();
        let mut state = unlocked(dir.path());
        let s = state.session_mut().unwrap();
        let m = s.save(None, None, "bye".into(), 100).unwrap();
        s.delete(&m.id).unwrap();
        assert!(s.list().is_empty());
        assert!(matches!(s.get(&m.id), Err(CoreError::NotFound)));
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
