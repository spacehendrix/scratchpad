use serde::{Deserialize, Serialize};

/// Timestamps are unix epoch milliseconds.
pub type UnixMs = i64;

/// A full document as the editor sees it.
#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct Document {
    /// UUIDv7 (time-ordered).
    pub id: String,
    pub title: Option<String>,
    pub body: String,
    pub meta: DocMeta,
}

/// Checklist progress derived from the body at save time, so list rendering
/// never needs to decrypt bodies.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct ChecklistCounts {
    pub done: u32,
    pub total: u32,
}

/// Listing/browse metadata — everything the UI needs without the body.
/// Stored encrypted as its own blob, decrypted into the in-memory catalog
/// on unlock.
#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct DocMeta {
    pub id: String,
    pub title: Option<String>,
    /// First non-empty line of the body, truncated; display name fallback.
    pub preview: String,
    pub created_at: UnixMs,
    /// Last content edit (edits and checkbox toggles only — opening a
    /// document does NOT refresh this). Drives the whole retention policy.
    pub updated_at: UnixMs,
    pub pinned: bool,
    pub archived_at: Option<UnixMs>,
    /// Size of the plaintext body in bytes.
    pub size_bytes: u32,
    pub checklist: Option<ChecklistCounts>,
}

#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct Settings {
    pub theme: String,
}

impl Default for Settings {
    fn default() -> Self {
        Settings {
            theme: "tokyo-night".into(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct StorageStats {
    pub db_bytes: u64,
    pub limit_bytes: u64,
    pub doc_count: u32,
    pub archived_count: u32,
    pub pinned_count: u32,
    /// True when over the limit but only pinned documents remain — saves are
    /// refused until the user unpins or deletes something.
    pub over_capacity: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct SearchHit {
    pub meta: DocMeta,
    /// Snippet around the first match (from title/preview or body).
    pub snippet: String,
    /// True when the match came from the body rather than title/preview.
    pub in_body: bool,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct RetentionReport {
    pub archived: u32,
    pub deleted_by_age: u32,
    pub deleted_by_space: u32,
    /// Destructive passes were skipped because system time regressed.
    pub skipped_clock_skew: bool,
}
