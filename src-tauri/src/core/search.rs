//! Search without an index: instant scan over the decrypted in-memory
//! catalog (titles/previews), then a streaming decrypt-and-scan over bodies.
//! Plaintext never persists — one body in memory at a time.

use crate::core::error::CoreResult;
use crate::core::model::SearchHit;
use crate::core::state::Session;

pub const MAX_HITS: usize = 100;
const CONTEXT_CHARS: usize = 40;

/// `scope_archived` selects which shelf is searched: the active documents
/// (false) or the archive (true) — mirroring the UI's scope toggle.
pub fn search(session: &Session, query: &str, scope_archived: bool) -> CoreResult<Vec<SearchHit>> {
    let needle = query.trim().to_lowercase();
    if needle.is_empty() {
        return Ok(Vec::new());
    }

    let in_scope = |archived_at: &Option<i64>| archived_at.is_some() == scope_archived;
    let mut hits: Vec<SearchHit> = Vec::new();

    // Phase 1: catalog (title + preview) — instant.
    for meta in &session.catalog {
        if !in_scope(&meta.archived_at) {
            continue;
        }
        let title_match = meta
            .title
            .as_deref()
            .is_some_and(|t| t.to_lowercase().contains(&needle));
        let preview_match = meta.preview.to_lowercase().contains(&needle);
        if title_match || preview_match {
            hits.push(SearchHit {
                meta: meta.clone(),
                snippet: meta.preview.clone(),
                in_body: false,
            });
        }
    }
    hits.sort_by_key(|h| -h.meta.updated_at);

    // Phase 2: bodies — streaming decrypt, skipping docs already hit.
    let already: Vec<&str> = hits.iter().map(|h| h.meta.id.as_str()).collect();
    let mut body_hits: Vec<SearchHit> = Vec::new();
    session.store.for_each_body(&session.key, |id, body| {
        if body_hits.len() + hits.len() >= MAX_HITS {
            return false;
        }
        if already.contains(&id) {
            return true;
        }
        let Some(meta) = session.catalog.iter().find(|m| m.id == id) else {
            return true;
        };
        if !in_scope(&meta.archived_at) {
            return true;
        }
        let lower = body.to_lowercase();
        if let Some(pos) = lower.find(&needle) {
            body_hits.push(SearchHit {
                meta: meta.clone(),
                snippet: snippet(body, pos, needle.len()),
                in_body: true,
            });
        }
        true
    })?;
    body_hits.sort_by_key(|h| -h.meta.updated_at);

    hits.extend(body_hits);
    hits.truncate(MAX_HITS);
    Ok(hits)
}

/// ±CONTEXT_CHARS of context around the match, char-boundary safe, newlines
/// flattened, ellipses when trimmed.
///
/// `start`/`len` are byte offsets into `text` (from a lowercase search over a
/// same-length string — lowercasing can shift byte offsets for exotic chars,
/// so the offsets are clamped to the nearest boundary rather than trusted).
pub fn snippet(text: &str, start: usize, len: usize) -> String {
    let clamp = |mut i: usize| {
        i = i.min(text.len());
        while i > 0 && !text.is_char_boundary(i) {
            i -= 1;
        }
        i
    };
    let match_start = clamp(start);
    let match_end = clamp(start + len);

    let before: String = text[..match_start]
        .chars()
        .rev()
        .take(CONTEXT_CHARS)
        .collect::<Vec<_>>()
        .into_iter()
        .rev()
        .collect();
    let after: String = text[match_end..].chars().take(CONTEXT_CHARS).collect();

    let mut s = String::new();
    if text[..match_start].chars().count() > CONTEXT_CHARS {
        s.push('…');
    }
    s.push_str(&before);
    s.push_str(&text[match_start..match_end]);
    s.push_str(&after);
    if text[match_end..].chars().count() > CONTEXT_CHARS {
        s.push('…');
    }
    s.split_whitespace().collect::<Vec<_>>().join(" ")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::keychain::InMemoryKeySource;
    use crate::core::state::AppState;

    fn unlocked(dir: &std::path::Path) -> AppState {
        let mut state = AppState::default();
        state.unlock(&InMemoryKeySource([1u8; 32]), dir).unwrap();
        state
    }

    #[test]
    fn finds_in_title_preview_and_body() {
        let dir = tempfile::tempdir().unwrap();
        let mut state = unlocked(dir.path());
        let s = state.session_mut().unwrap();
        s.save(None, Some("Groceries".into()), "eggs\nmilk".into(), 1).unwrap();
        s.save(None, None, "band practice riff".into(), 2).unwrap();
        s.save(None, None, "first line\ndeep in the body: groceries too".into(), 3).unwrap();

        let hits = search(s, "groceries", false).unwrap();
        assert_eq!(hits.len(), 2);
        // Catalog hit (title) ranks before body hit.
        assert!(!hits[0].in_body);
        assert_eq!(hits[0].meta.title.as_deref(), Some("Groceries"));
        assert!(hits[1].in_body);
        assert!(hits[1].snippet.contains("groceries too"));
    }

    #[test]
    fn search_is_case_insensitive() {
        let dir = tempfile::tempdir().unwrap();
        let mut state = unlocked(dir.path());
        let s = state.session_mut().unwrap();
        s.save(None, None, "Buy MILK today".into(), 1).unwrap();
        assert_eq!(search(s, "milk", false).unwrap().len(), 1);
        assert_eq!(search(s, "MILK", false).unwrap().len(), 1);
    }

    #[test]
    fn scope_separates_active_and_archive() {
        let dir = tempfile::tempdir().unwrap();
        let mut state = unlocked(dir.path());
        let s = state.session_mut().unwrap();
        s.save(None, None, "active note".into(), 1).unwrap();
        let old = s.save(None, None, "archived note".into(), 2).unwrap();
        let mut archived = old.clone();
        archived.archived_at = Some(100);
        s.store.update_meta(&s.key, &archived).unwrap();
        *s.catalog.iter_mut().find(|m| m.id == old.id).unwrap() = archived;

        let active = search(s, "note", false).unwrap();
        assert_eq!(active.len(), 1);
        assert_eq!(active[0].meta.preview, "active note");

        let archive = search(s, "note", true).unwrap();
        assert_eq!(archive.len(), 1);
        assert_eq!(archive[0].meta.preview, "archived note");
    }

    #[test]
    fn empty_query_returns_nothing() {
        let dir = tempfile::tempdir().unwrap();
        let mut state = unlocked(dir.path());
        let s = state.session_mut().unwrap();
        s.save(None, None, "something".into(), 1).unwrap();
        assert!(search(s, "", false).unwrap().is_empty());
        assert!(search(s, "   ", false).unwrap().is_empty());
    }

    #[test]
    fn snippet_trims_flattens_and_marks_ellipses() {
        let text = format!("{}needle{}", "a".repeat(100), "b".repeat(100));
        let pos = text.find("needle").unwrap();
        let s = snippet(&text, pos, 6);
        assert!(s.starts_with('…'));
        assert!(s.ends_with('…'));
        assert!(s.contains("needle"));

        let multi = "line one\nline two with needle here\nline three";
        let s = snippet(multi, multi.find("needle").unwrap(), 6);
        assert!(!s.contains('\n'));
        assert!(s.contains("needle here"));
    }

    #[test]
    fn snippet_survives_multibyte_boundaries() {
        let text = "🎸🎸🎸 émotion cœur naïve 🎸🎸🎸";
        let pos = text.find("cœur").unwrap();
        let s = snippet(text, pos, "cœur".len());
        assert!(s.contains("cœur"));
        // Deliberately mid-codepoint offsets must not panic.
        let _ = snippet(text, 1, 2);
        let _ = snippet(text, text.len() - 1, 10);
    }
}
