//! Encrypted SQLite persistence. The database stores only ciphertext:
//! each document row is `(id, meta_blob, body_blob)` where both blobs are
//! sealed with the master key and bound to `{id}:meta` / `{id}:body` AAD.

use std::path::{Path, PathBuf};

use rusqlite::{params, Connection, OptionalExtension};

use crate::core::crypto::{aad, open, seal, MasterKey};
use crate::core::error::{CoreError, CoreResult};
use crate::core::model::DocMeta;

pub const DB_FILE: &str = "scratchpad.db";
const SCHEMA_VERSION: i32 = 1;

pub struct Store {
    conn: Connection,
    db_path: PathBuf,
}

impl Store {
    /// Open (creating if needed) the database inside `dir`.
    pub fn open(dir: &Path) -> CoreResult<Store> {
        std::fs::create_dir_all(dir)?;
        Self::open_at(&dir.join(DB_FILE))
    }

    fn open_at(db_path: &Path) -> CoreResult<Store> {
        let conn = Connection::open(db_path).map_err(sql_err)?;
        // auto_vacuum must be configured before the first table is created.
        conn.pragma_update(None, "auto_vacuum", "INCREMENTAL")
            .map_err(sql_err)?;
        conn.pragma_update(None, "journal_mode", "WAL")
            .map_err(sql_err)?;
        conn.pragma_update(None, "synchronous", "NORMAL")
            .map_err(sql_err)?;

        let version: i32 = conn
            .query_row("PRAGMA user_version", [], |r| r.get(0))
            .map_err(sql_err)?;
        if version < SCHEMA_VERSION {
            conn.execute_batch(
                "BEGIN;
                 CREATE TABLE IF NOT EXISTS documents (
                   id        TEXT PRIMARY KEY,
                   meta_blob BLOB NOT NULL,
                   body_blob BLOB NOT NULL
                 );
                 PRAGMA user_version = 1;
                 COMMIT;",
            )
            .map_err(sql_err)?;
        }

        Ok(Store {
            conn,
            db_path: db_path.to_path_buf(),
        })
    }

    /// Insert or update a document — both blobs written in one transaction,
    /// so a crash mid-save can never leave meta and body out of sync.
    pub fn upsert(&mut self, key: &MasterKey, meta: &DocMeta, body: &str) -> CoreResult<()> {
        let meta_json = serde_json::to_vec(meta).map_err(|e| CoreError::Io(e.to_string()))?;
        let meta_blob = seal(key, &aad(&meta.id, "meta"), &meta_json);
        let body_blob = seal(key, &aad(&meta.id, "body"), body.as_bytes());

        let tx = self.conn.transaction().map_err(sql_err)?;
        tx.execute(
            "INSERT INTO documents (id, meta_blob, body_blob) VALUES (?1, ?2, ?3)
             ON CONFLICT(id) DO UPDATE SET meta_blob = ?2, body_blob = ?3",
            params![meta.id, meta_blob, body_blob],
        )
        .map_err(sql_err)?;
        tx.commit().map_err(sql_err)
    }

    /// Decrypt every document's metadata into the in-memory catalog.
    pub fn load_catalog(&self, key: &MasterKey) -> CoreResult<Vec<DocMeta>> {
        let mut stmt = self
            .conn
            .prepare("SELECT id, meta_blob FROM documents")
            .map_err(sql_err)?;
        let rows = stmt
            .query_map([], |r| {
                Ok((r.get::<_, String>(0)?, r.get::<_, Vec<u8>>(1)?))
            })
            .map_err(sql_err)?;

        let mut catalog = Vec::new();
        for row in rows {
            let (id, blob) = row.map_err(sql_err)?;
            let plain = open(key, &aad(&id, "meta"), &blob)
                .map_err(|_| CoreError::Corrupt { id: id.clone() })?;
            let meta: DocMeta =
                serde_json::from_slice(&plain).map_err(|_| CoreError::Corrupt { id: id.clone() })?;
            catalog.push(meta);
        }
        Ok(catalog)
    }

    pub fn load_body(&self, key: &MasterKey, id: &str) -> CoreResult<String> {
        let blob: Option<Vec<u8>> = self
            .conn
            .query_row(
                "SELECT body_blob FROM documents WHERE id = ?1",
                params![id],
                |r| r.get(0),
            )
            .optional()
            .map_err(sql_err)?;
        let blob = blob.ok_or(CoreError::NotFound)?;
        let plain =
            open(key, &aad(id, "body"), &blob).map_err(|_| CoreError::Corrupt { id: id.into() })?;
        String::from_utf8(plain).map_err(|_| CoreError::Corrupt { id: id.into() })
    }

    /// Visit every body one at a time (streaming, for search) without holding
    /// all plaintext in memory. The visitor returns `false` to stop early.
    pub fn for_each_body(
        &self,
        key: &MasterKey,
        mut visit: impl FnMut(&str, &str) -> bool,
    ) -> CoreResult<()> {
        let mut stmt = self
            .conn
            .prepare("SELECT id, body_blob FROM documents")
            .map_err(sql_err)?;
        let rows = stmt
            .query_map([], |r| {
                Ok((r.get::<_, String>(0)?, r.get::<_, Vec<u8>>(1)?))
            })
            .map_err(sql_err)?;
        for row in rows {
            let (id, blob) = row.map_err(sql_err)?;
            // A single corrupt row shouldn't break search over the rest.
            if let Ok(plain) = open(key, &aad(&id, "body"), &blob) {
                if let Ok(text) = String::from_utf8(plain) {
                    if !visit(&id, &text) {
                        break;
                    }
                }
            }
        }
        Ok(())
    }

    pub fn delete(&mut self, id: &str) -> CoreResult<()> {
        let n = self
            .conn
            .execute("DELETE FROM documents WHERE id = ?1", params![id])
            .map_err(sql_err)?;
        if n == 0 {
            return Err(CoreError::NotFound);
        }
        Ok(())
    }

    pub fn delete_many(&mut self, ids: &[String]) -> CoreResult<()> {
        let tx = self.conn.transaction().map_err(sql_err)?;
        for id in ids {
            tx.execute("DELETE FROM documents WHERE id = ?1", params![id])
                .map_err(sql_err)?;
        }
        tx.commit().map_err(sql_err)
    }

    /// On-disk footprint: db file + WAL (what the 5 GB cap is measured on).
    pub fn db_size_bytes(&self) -> u64 {
        let file = |p: &Path| std::fs::metadata(p).map(|m| m.len()).unwrap_or(0);
        let mut wal = self.db_path.as_os_str().to_owned();
        wal.push("-wal");
        file(&self.db_path) + file(Path::new(&wal))
    }

    /// Reclaim space after bulk deletions.
    pub fn vacuum(&self) -> CoreResult<()> {
        self.conn
            .execute_batch("PRAGMA incremental_vacuum; PRAGMA wal_checkpoint(TRUNCATE);")
            .map_err(sql_err)
    }
}

fn sql_err(e: rusqlite::Error) -> CoreError {
    CoreError::Io(e.to_string())
}

/// Move an undecryptable database aside instead of deleting it ("start
/// fresh" flow after the keychain key was lost). Reversible by hand.
pub fn quarantine_db(data_dir: &Path, suffix: &str) -> CoreResult<()> {
    for ext in ["", "-wal", "-shm"] {
        let src = data_dir.join(format!("{DB_FILE}{ext}"));
        if src.exists() {
            let dst = data_dir.join(format!("{DB_FILE}.corrupt-{suffix}{ext}"));
            std::fs::rename(&src, &dst)?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::model::{derive_checklist, derive_preview};

    fn test_key() -> MasterKey {
        MasterKey::from_bytes([7u8; 32])
    }

    fn meta(id: &str, body: &str) -> DocMeta {
        DocMeta {
            id: id.into(),
            title: None,
            preview: derive_preview(body),
            created_at: 1_000,
            updated_at: 1_000,
            pinned: false,
            archived_at: None,
            size_bytes: body.len() as u32,
            checklist: derive_checklist(body),
        }
    }

    #[test]
    fn crud_roundtrip_and_reopen() {
        let dir = tempfile::tempdir().unwrap();
        let key = test_key();
        let body = "buy milk\n[ ] eggs\n[x] bread";
        {
            let mut store = Store::open(dir.path()).unwrap();
            store.upsert(&key, &meta("doc-1", body), body).unwrap();

            let catalog = store.load_catalog(&key).unwrap();
            assert_eq!(catalog.len(), 1);
            assert_eq!(catalog[0].preview, "buy milk");
            assert_eq!(catalog[0].checklist.unwrap().total, 2);
            assert_eq!(catalog[0].checklist.unwrap().done, 1);
            assert_eq!(store.load_body(&key, "doc-1").unwrap(), body);
        }
        // Reopen from disk — everything persisted.
        let store = Store::open(dir.path()).unwrap();
        assert_eq!(store.load_body(&key, "doc-1").unwrap(), body);
    }

    #[test]
    fn update_replaces_both_blobs() {
        let dir = tempfile::tempdir().unwrap();
        let key = test_key();
        let mut store = Store::open(dir.path()).unwrap();
        store.upsert(&key, &meta("doc-1", "v1"), "v1").unwrap();
        store.upsert(&key, &meta("doc-1", "v2"), "v2").unwrap();
        assert_eq!(store.load_body(&key, "doc-1").unwrap(), "v2");
        let catalog = store.load_catalog(&key).unwrap();
        assert_eq!(catalog.len(), 1);
        assert_eq!(catalog[0].preview, "v2");
    }

    #[test]
    fn delete_and_not_found() {
        let dir = tempfile::tempdir().unwrap();
        let key = test_key();
        let mut store = Store::open(dir.path()).unwrap();
        store.upsert(&key, &meta("doc-1", "x"), "x").unwrap();
        store.delete("doc-1").unwrap();
        assert!(matches!(
            store.load_body(&key, "doc-1"),
            Err(CoreError::NotFound)
        ));
        assert!(matches!(store.delete("doc-1"), Err(CoreError::NotFound)));
    }

    #[test]
    fn no_plaintext_on_disk() {
        let dir = tempfile::tempdir().unwrap();
        let key = test_key();
        let sentinel = "SENTINEL_PLAINTEXT_8f3a";
        {
            let mut store = Store::open(dir.path()).unwrap();
            let body = format!("top secret {sentinel} content");
            let mut m = meta("doc-1", &body);
            m.title = Some(format!("title {sentinel}"));
            store.upsert(&key, &m, &body).unwrap();
            store.vacuum().unwrap(); // checkpoint WAL into the main db file
        }
        for entry in std::fs::read_dir(dir.path()).unwrap() {
            let path = entry.unwrap().path();
            let bytes = std::fs::read(&path).unwrap();
            assert!(
                !bytes
                    .windows(sentinel.len())
                    .any(|w| w == sentinel.as_bytes()),
                "plaintext sentinel leaked into {path:?}"
            );
        }
    }

    #[test]
    fn corrupt_meta_blob_is_reported_with_id() {
        let dir = tempfile::tempdir().unwrap();
        let key = test_key();
        let mut store = Store::open(dir.path()).unwrap();
        store.upsert(&key, &meta("doc-1", "x"), "x").unwrap();
        store
            .conn
            .execute(
                "UPDATE documents SET meta_blob = x'00112233' WHERE id = 'doc-1'",
                [],
            )
            .unwrap();
        match store.load_catalog(&key) {
            Err(CoreError::Corrupt { id }) => assert_eq!(id, "doc-1"),
            other => panic!("expected Corrupt, got {other:?}"),
        }
    }

    #[test]
    fn for_each_body_streams_and_stops() {
        let dir = tempfile::tempdir().unwrap();
        let key = test_key();
        let mut store = Store::open(dir.path()).unwrap();
        for i in 0..5 {
            let id = format!("doc-{i}");
            let body = format!("body {i}");
            store.upsert(&key, &meta(&id, &body), &body).unwrap();
        }
        let mut seen = 0;
        store
            .for_each_body(&key, |_, _| {
                seen += 1;
                seen < 3
            })
            .unwrap();
        assert_eq!(seen, 3);
    }

    #[test]
    fn wrong_key_reports_corrupt() {
        let dir = tempfile::tempdir().unwrap();
        let mut store = Store::open(dir.path()).unwrap();
        store.upsert(&test_key(), &meta("doc-1", "x"), "x").unwrap();
        let wrong = MasterKey::from_bytes([9u8; 32]);
        assert!(matches!(
            store.load_body(&wrong, "doc-1"),
            Err(CoreError::Corrupt { .. })
        ));
    }
}
